use std::{
    collections::{btree_set::Iter, BTreeMap, BTreeSet, HashSet},
    error::Error,
    time::SystemTime,
};

use candid::{Deserialize, Principal};
use ic_cdk::api::{call::CallResult, management_canister::provisional::CanisterId};
use ic_stable_structures::btreemap::BTreeMap as StableBTreeMap;
use memory::{
    get_bet_details_memory_v2, get_post_principal_memory_v2, get_room_details_memory_v2,
    get_slot_details_memory_v2, get_success_history_memory, get_token_list_memory,
    get_watch_history_memory,
};
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        cdao::DeployedCdaoCanisters,
        configuration::IndividualUserConfiguration,
        device_id::DeviceIdentity,
        error::{BetOnCurrentlyViewingPostError, GetPostsOfUserProfileError},
        follow::FollowData,
        hot_or_not::{
            BetDetails, BetDirection, BetOutcomeForBetMaker, BettingStatus, GlobalBetId,
            GlobalRoomId, HotOrNotGame, PlacedBetDetail, RoomDetailsV1, SlotDetailsV1, SlotId,
            StablePrincipal,
        },
        migration::MigrationInfo,
        ml_data::{
            MLData, MLFeedCacheItem, SuccessHistoryItem, SuccessHistoryItemV1, WatchHistoryItem,
        },
        post::{Post, PostDetailsForFrontend, PostDetailsFromFrontend},
        profile::{UserProfile, UserProfileDetailsForFrontend},
        session::SessionType,
        token::{TokenBalance, TokenTransactions},
    },
    common::{
        types::{
            app_primitive_type::PostId,
            known_principal::KnownPrincipalMap,
            top_posts::{post_score_index::PostScoreIndex, post_score_index_item::PostStatus},
            utility_token::token_event::{HotOrNotOutcomePayoutEvent, StakeEvent, TokenEvent},
            version_details::VersionDetails,
        },
        utils::system_time::{self, get_current_system_time},
    },
    pagination::{self, PaginationError},
};

use crate::api::post;

use self::memory::{
    get_bet_details_memory, get_post_principal_memory, get_room_details_memory,
    get_slot_details_memory, Memory,
};

use kv_storage::AppStorage;

pub mod cents_hot_or_not_game;
pub mod kv_storage;
pub mod memory;
pub mod pump_n_dump;

impl HotOrNotGame for CanisterData {
    fn prepare_for_bet(
        &mut self,
        bet_marker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
        current_timestamp: SystemTime,
    ) -> Result<(), BetOnCurrentlyViewingPostError> {
        self.validate_incoming_bet(bet_marker_principal, &place_bet_arg)?;

        self.my_token_balance.handle_token_event(TokenEvent::Stake {
            amount: place_bet_arg.bet_amount,
            details: StakeEvent::BetOnHotOrNotPost {
                post_canister_id: place_bet_arg.post_canister_id,
                post_id: place_bet_arg.post_id,
                bet_amount: place_bet_arg.bet_amount,
                bet_direction: place_bet_arg.bet_direction,
            },
            timestamp: current_timestamp,
        });

        Ok(())
    }
    fn process_place_bet_status(
        &mut self,
        bet_response: CallResult<(Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>,
        place_bet_arg: &PlaceBetArg,
        current_timestamp: SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let bet_response = bet_response
            .map_err(|_e| BetOnCurrentlyViewingPostError::PostCreatorCanisterCallFailed)
            .map(|res| res.0)
            .and_then(|inner| inner)
            .inspect_err(|_| {
                self.my_token_balance.handle_token_event(TokenEvent::Stake {
                    amount: place_bet_arg.bet_amount,
                    details: StakeEvent::BetFailureRefund {
                        bet_amount: place_bet_arg.bet_amount,
                        post_id: place_bet_arg.post_id,
                        post_canister_id: place_bet_arg.post_canister_id,
                        bet_direction: place_bet_arg.bet_direction,
                    },
                    timestamp: current_timestamp,
                });
            })?;

        match bet_response {
            BettingStatus::BettingClosed => {
                self.my_token_balance.handle_token_event(TokenEvent::Stake {
                    amount: place_bet_arg.bet_amount,
                    details: StakeEvent::BetFailureRefund {
                        bet_amount: place_bet_arg.bet_amount,
                        post_id: place_bet_arg.post_id,
                        post_canister_id: place_bet_arg.post_canister_id,
                        bet_direction: place_bet_arg.bet_direction,
                    },
                    timestamp: get_current_system_time(),
                });
                return Err(BetOnCurrentlyViewingPostError::BettingClosed);
            }
            BettingStatus::BettingOpen {
                ongoing_slot,
                ongoing_room,
                ..
            } => {
                let all_hot_or_not_bets_placed = &mut self.all_hot_or_not_bets_placed;
                all_hot_or_not_bets_placed.insert(
                    (place_bet_arg.post_canister_id, place_bet_arg.post_id),
                    PlacedBetDetail {
                        canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        slot_id: ongoing_slot,
                        room_id: ongoing_room,
                        bet_direction: place_bet_arg.bet_direction,
                        bet_placed_at: current_timestamp,
                        amount_bet: place_bet_arg.bet_amount,
                        outcome_received: BetOutcomeForBetMaker::default(),
                    },
                );
            }
        }

        Ok(bet_response)
    }

    fn validate_incoming_bet(
        &self,
        bet_maker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
    ) -> Result<(), BetOnCurrentlyViewingPostError> {
        if bet_maker_principal == Principal::anonymous() {
            return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
        }

        let profile_owner = self
            .profile
            .principal_id
            .ok_or(BetOnCurrentlyViewingPostError::UserPrincipalNotSet)?;

        if bet_maker_principal != profile_owner {
            return Err(BetOnCurrentlyViewingPostError::Unauthorized);
        }

        let utlility_token_balance = self.my_token_balance.get_current_token_balance();

        if utlility_token_balance < place_bet_arg.bet_amount as u128 {
            return Err(BetOnCurrentlyViewingPostError::InsufficientBalance);
        }

        if self
            .all_hot_or_not_bets_placed
            .contains_key(&(place_bet_arg.post_canister_id, place_bet_arg.post_id))
        {
            return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
        }

        Ok(())
    }

    fn receive_bet_from_bet_maker_canister(
        &mut self,
        bet_maker_principal_id: Principal,
        bet_maker_canister_id: Principal,
        place_bet_arg: &PlaceBetArg,
        current_timestamp: SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let PlaceBetArg { post_id, .. } = place_bet_arg;

        self.register_hot_or_not_bet_for_post(
            *post_id,
            bet_maker_principal_id,
            bet_maker_canister_id,
            place_bet_arg,
            &current_timestamp,
        )
    }

    fn tabulate_hot_or_not_outcome_for_post_slot(
        &mut self,
        post_id: u64,
        slot_id: u8,
        current_timestamp: SystemTime,
    ) {
        let post_res = CanisterData::get_post_mut(&mut self.all_created_posts, post_id);

        let Some(post) = post_res else {
            return;
        };

        post.tabulate_hot_or_not_outcome_for_slot_v1(
            &ic_cdk::id(),
            &slot_id,
            &mut self.my_token_balance,
            &current_timestamp,
            &mut self.room_details_map,
            &mut self.bet_details_map,
        );

        self.all_created_posts
            .get_mut(&post_id)
            .map(|post| post.slots_left_to_be_computed.remove(&slot_id));
    }

    fn receive_earnings_for_the_bet(
        &mut self,
        post_id: u64,
        post_creator_canister_id: Principal,
        outcome: BetOutcomeForBetMaker,
        current_timestamp: SystemTime,
    ) {
        if !self
            .all_hot_or_not_bets_placed
            .contains_key(&(post_creator_canister_id, post_id))
        {
            return;
        }

        if self
            .all_hot_or_not_bets_placed
            .get(&(post_creator_canister_id, post_id))
            .unwrap()
            .outcome_received
            != BetOutcomeForBetMaker::AwaitingResult
        {
            return;
        }

        let all_hot_or_not_bets_placed = &mut self.all_hot_or_not_bets_placed;

        all_hot_or_not_bets_placed
            .entry((post_creator_canister_id, post_id))
            .and_modify(|placed_bet_detail| {
                placed_bet_detail.outcome_received = outcome.clone();
            });

        let placed_bet_detail = all_hot_or_not_bets_placed
            .get(&(post_creator_canister_id, post_id))
            .cloned()
            .unwrap();

        self.my_token_balance
            .handle_token_event(TokenEvent::HotOrNotOutcomePayout {
                amount: match outcome {
                    BetOutcomeForBetMaker::Draw(amount) => amount,
                    BetOutcomeForBetMaker::Won(amount) => amount,
                    _ => 0,
                },
                details: HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                    post_canister_id: post_creator_canister_id,
                    post_id,
                    slot_id: placed_bet_detail.slot_id,
                    room_id: placed_bet_detail.room_id,
                    winnings_amount: match outcome {
                        BetOutcomeForBetMaker::Draw(amount) => amount,
                        BetOutcomeForBetMaker::Won(amount) => amount,
                        _ => 0,
                    },
                    event_outcome: outcome,
                },
                timestamp: current_timestamp,
            });
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct CanisterData {
    // Key is Post ID
    all_created_posts: BTreeMap<u64, Post>,
    #[serde(skip, default = "_default_room_details")]
    pub room_details_map:
        ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory>,
    #[serde(skip, default = "_default_bet_details")]
    pub bet_details_map: ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory>,
    #[serde(skip, default = "_default_post_principal_map")]
    pub post_principal_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>,
    #[serde(skip, default = "_default_slot_details_map")]
    pub slot_details_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory>,
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
    pub configuration: IndividualUserConfiguration,
    pub follow_data: FollowData,
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    #[serde(default)]
    pub session_type: Option<SessionType>,
    #[serde(default)]
    pub last_access_time: Option<SystemTime>,
    #[serde(default)]
    pub last_canister_functionality_access_time: Option<SystemTime>,
    #[serde(default)]
    pub migration_info: MigrationInfo,
    #[serde(default)]
    pub app_storage: AppStorage,
    #[serde(skip, default = "_default_watch_history")]
    pub watch_history: ic_stable_structures::btreemap::BTreeMap<WatchHistoryItem, (), Memory>,
    #[serde(skip, default = "_default_success_history_v1")]
    pub success_history: ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItemV1, (), Memory>,
    #[serde(default)]
    pub cdao_canisters: Vec<DeployedCdaoCanisters>,
    // list of root token canisters
    #[serde(skip, default = "_default_token_list")]
    pub token_roots: ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory>,
    #[serde(default)]
    pub empty_canisters: AllotedEmptyCanister,
}

impl CanisterData {
    pub(crate) fn delete_post(&mut self, post_id: u64) -> Result<(), String> {
        let post = self
            .all_created_posts
            .get_mut(&post_id)
            .ok_or("Post not found".to_owned())?;

        match post.status {
            PostStatus::Deleted => Err("Post not found".to_owned()),
            _ => {
                post.status = PostStatus::Deleted;
                Ok(())
            }
        }
    }

    pub fn set_all_created_posts(&mut self, all_created_post: BTreeMap<u64, Post>) {
        self.all_created_posts = all_created_post;
    }

    pub fn get_all_posts_cloned(&self) -> Vec<(u64, Post)> {
        self.all_created_posts
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    pub fn add_post(&mut self, post: Post) -> Option<Post> {
        self.all_created_posts.insert(post.id, post)
    }

    pub fn contains_post(&self, post_id: &u64) -> bool {
        self.all_created_posts.contains_key(post_id)
    }

    pub fn add_post_to_memory(
        &mut self,
        post_details_from_frontend: &PostDetailsFromFrontend,
        current_time: &SystemTime,
    ) -> u64 {
        let post_id = self.all_created_posts.len() as u64;
        self.add_post(Post::new(post_id, post_details_from_frontend, current_time));

        post_id
    }

    pub fn get_posts_with_pagination_cursor(
        &self,
        from_inclusive_index: u64,
        limit: u64,
        api_caller_principal_id: Principal,
        current_time: SystemTime,
    ) -> Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            self.all_created_posts.len() as u64,
        )
        .map_err(|e| match e {
            PaginationError::InvalidBoundsPassed => GetPostsOfUserProfileError::InvalidBoundsPassed,
            PaginationError::ReachedEndOfItemsList => {
                GetPostsOfUserProfileError::ReachedEndOfItemsList
            }
            PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
                GetPostsOfUserProfileError::ExceededMaxNumberOfItemsAllowedInOneRequest
            }
        })?;

        let res_posts = self
            .all_created_posts
            .iter()
            .filter(|(_, post)| {
                post.status != PostStatus::BannedDueToUserReporting
                    && post.status != PostStatus::Deleted
            })
            .rev()
            .skip(from_inclusive_index as usize)
            .take(limit as usize)
            .map(|(id, post)| {
                let profile = &self.profile;
                let followers = &self.principals_that_follow_me;
                let following = &self.principals_i_follow;
                let token_balance = &self.my_token_balance;

                post.get_post_details_for_frontend_for_this_post(
                    UserProfileDetailsForFrontend {
                        display_name: profile.display_name.clone(),
                        followers_count: followers.len() as u64,
                        following_count: following.len() as u64,
                        principal_id: profile.principal_id.unwrap(),
                        profile_picture_url: profile.profile_picture_url.clone(),
                        profile_stats: profile.profile_stats,
                        unique_user_name: profile.unique_user_name.clone(),
                        lifetime_earnings: token_balance.lifetime_earnings,
                        referrer_details: profile.referrer_details.clone(),
                    },
                    api_caller_principal_id,
                    &current_time,
                    &self.room_details_map,
                    &self.post_principal_map,
                    &self.slot_details_map,
                )
            })
            .collect();

        Ok(res_posts)
    }

    pub fn get_posts_that_have_pending_outcomes(&self) -> Vec<u64> {
        self.all_created_posts
            .iter()
            .filter(|(_post_id, post)| {
                !post.slots_left_to_be_computed.is_empty() && post.status != PostStatus::Deleted
            })
            .map(|(post_id, _post)| *post_id)
            .collect()
    }

    pub fn register_hot_or_not_bet_for_post(
        &mut self,
        post_id: u64,
        bet_maker_principal_id: Principal,
        bet_maker_canister_id: Principal,
        place_bet_arg: &PlaceBetArg,
        current_time_when_request_being_made: &SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let post = CanisterData::get_post_mut(&mut self.all_created_posts, post_id).unwrap();
        let PlaceBetArg {
            bet_amount,
            bet_direction,
            ..
        } = place_bet_arg;
        let betting_status = post.place_hot_or_not_bet_v1(
            &bet_maker_principal_id,
            &bet_maker_canister_id,
            *bet_amount,
            bet_direction,
            current_time_when_request_being_made,
            &mut self.room_details_map,
            &mut self.bet_details_map,
            &mut self.post_principal_map,
            &mut self.slot_details_map,
        )?;

        match *bet_direction {
            BetDirection::Hot => {
                self.profile.profile_stats.hot_bets_received += 1;
            }
            BetDirection::Not => {
                self.profile.profile_stats.not_bets_received += 1;
            }
        }

        Ok(betting_status)
    }

    fn get_post_mut(posts: &mut BTreeMap<u64, Post>, post_id: u64) -> Option<&mut Post> {
        posts.get_mut(&post_id).and_then(|post| match post.status {
            PostStatus::Deleted => None,
            _ => Some(post),
        })
    }

    pub fn get_post(&self, post_id: &u64) -> Option<&Post> {
        self.all_created_posts
            .get(post_id)
            .and_then(|post| match post.status {
                PostStatus::Deleted => None,
                _ => Some(post),
            })
    }

    pub(crate) fn get_post_for_frontend(
        &self,
        post_id: u64,
        caller: Principal,
    ) -> PostDetailsForFrontend {
        let post = self.get_post(&post_id).unwrap();
        let profile = &self.profile;
        let followers = &self.principals_that_follow_me;
        let following = &self.principals_i_follow;
        let token_balance = &self.my_token_balance;

        post.get_post_details_for_frontend_for_this_post(
            UserProfileDetailsForFrontend {
                display_name: profile.display_name.clone(),
                followers_count: followers.len() as u64,
                following_count: following.len() as u64,
                principal_id: profile.principal_id.unwrap(),
                profile_picture_url: profile.profile_picture_url.clone(),
                profile_stats: profile.profile_stats,
                unique_user_name: profile.unique_user_name.clone(),
                lifetime_earnings: token_balance.lifetime_earnings,
                referrer_details: profile.referrer_details.clone(),
            },
            caller,
            &system_time::get_current_system_time_from_ic(),
            &self.room_details_map,
            &self.post_principal_map,
            &self.slot_details_map,
        )
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AllotedEmptyCanister {
    canister_ids: HashSet<Principal>,
}

impl AllotedEmptyCanister {
    pub fn get_number_of_canister(&mut self, number: usize) -> Result<Vec<Principal>, String> {
        let mut canister_ids = vec![];
        let mut iterator = self.canister_ids.iter().copied();
        for _ in 0..number {
            if let Some(canister_id) = iterator.next() {
                canister_ids.push(canister_id);
            } else {
                return Err(format!("{} number of canisters not available", number));
            }
        }

        self.canister_ids = iterator.collect();

        Ok(canister_ids)
    }

    pub fn insert_empty_canister(&mut self, canister_id: Principal) -> bool {
        self.canister_ids.insert(canister_id)
    }

    pub fn append_empty_canisters(&mut self, canister_ids: Vec<Principal>) {
        self.canister_ids.extend(canister_ids.into_iter());
    }

    pub fn len(&self) -> usize {
        self.canister_ids.len()
    }
}

pub fn _default_room_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory())
}

pub fn _default_bet_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory())
}

pub fn _default_post_principal_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory())
}

pub fn _default_slot_details_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_slot_details_memory())
}

pub fn _default_watch_history(
) -> ic_stable_structures::btreemap::BTreeMap<WatchHistoryItem, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_watch_history_memory())
}

#[deprecated(note = "Use _default_success_history_v1 instead")]
pub fn _default_success_history(
) -> ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItem, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_success_history_memory())
}

pub fn _default_token_list() -> ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_token_list_memory())
}

pub fn _default_success_history_v1(
) -> ic_stable_structures::btreemap::BTreeMap<SuccessHistoryItemV1, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_success_history_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            all_created_posts: BTreeMap::new(),
            room_details_map: _default_room_details(),
            bet_details_map: _default_bet_details(),
            post_principal_map: _default_post_principal_map(),
            slot_details_map: _default_slot_details_map(),
            all_hot_or_not_bets_placed: BTreeMap::new(),
            configuration: IndividualUserConfiguration::default(),
            follow_data: FollowData::default(),
            known_principal_ids: KnownPrincipalMap::default(),
            my_token_balance: TokenBalance::default(),
            principals_i_follow: BTreeSet::new(),
            principals_that_follow_me: BTreeSet::new(),
            profile: UserProfile::default(),
            version_details: VersionDetails::default(),
            session_type: None,
            last_access_time: None,
            last_canister_functionality_access_time: None,
            migration_info: MigrationInfo::NotMigrated,
            app_storage: AppStorage::default(),
            watch_history: _default_watch_history(),
            success_history: _default_success_history_v1(),
            cdao_canisters: Vec::new(),
            token_roots: _default_token_list(),
            empty_canisters: AllotedEmptyCanister::default(),
        }
    }
}
