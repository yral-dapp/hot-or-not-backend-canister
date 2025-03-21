use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    time::SystemTime,
};

use candid::{Deserialize, Principal};
use ic_cdk::api::{call::CallResult, management_canister::provisional::CanisterId};
use memory::{get_success_history_memory, get_token_list_memory, get_watch_history_memory};
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        cdao::DeployedCdaoCanisters,
        configuration::IndividualUserConfiguration,
        device_id::DeviceIdentity,
        error::BetOnCurrentlyViewingPostError,
        follow::FollowData,
        hot_or_not::{
            BetDetails, BetOutcomeForBetMaker, BettingStatus, GlobalBetId, GlobalRoomId,
            PlacedBetDetail, RoomDetailsV1, SlotDetailsV1, SlotId, StablePrincipal,
        },
        migration::MigrationInfo,
        ml_data::{
            MLData, MLFeedCacheItem, SuccessHistoryItem, SuccessHistoryItemV1, WatchHistoryItem,
        },
        post::{Post, PostDetailsForFrontend},
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
};

use crate::api::post;

use self::memory::{
    get_bet_details_memory, get_post_principal_memory, get_room_details_memory,
    get_slot_details_memory, Memory,
};

use kv_storage::AppStorage;

pub mod kv_storage;
pub mod memory;
pub mod pump_n_dump;

pub(crate) trait HotOrNotGame {
    fn validate_incoming_bet(
        &self,
        token: &dyn TokenTransactions,
        bet_maker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
    ) -> Result<(), BetOnCurrentlyViewingPostError>;
    fn prepare_for_bet(
        &mut self,
        bet_maker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
        token: &mut dyn TokenTransactions,
        current_timestamp: SystemTime,
    ) -> Result<(), BetOnCurrentlyViewingPostError>;

    fn process_place_bet_status(
        &mut self,
        bet_response: CallResult<(Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>,
        place_bet_arg: &PlaceBetArg,
        token: &mut dyn TokenTransactions,
        current_timestamp: SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError>;
    fn receive_earnings_for_the_bet(
        &mut self,
        earnings_amount: u128,
        hot_or_not_outcome_details: HotOrNotOutcomePayoutEvent,
        timestamp: SystemTime,
    );
}

impl HotOrNotGame for CanisterData {
    fn prepare_for_bet(
        &mut self,
        bet_marker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
        token: &mut dyn TokenTransactions,
        current_timestamp: SystemTime,
    ) -> Result<(), BetOnCurrentlyViewingPostError> {
        self.validate_incoming_bet(token, bet_marker_principal, &place_bet_arg)?;

        token.handle_token_event(TokenEvent::Stake {
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
        token: &mut dyn TokenTransactions,
        current_timestamp: SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let bet_response = bet_response
            .map_err(|e| BetOnCurrentlyViewingPostError::PostCreatorCanisterCallFailed)
            .map(|res| res.0)
            .and_then(|inner| inner)
            .inspect_err(|_| {
                token.handle_token_event(TokenEvent::Stake {
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
                token.handle_token_event(TokenEvent::Stake {
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

    fn receive_earnings_for_the_bet(
        &mut self,
        earnings_amount: u128,
        hot_or_not_outcome_details: HotOrNotOutcomePayoutEvent,
        timestamp: SystemTime,
    ) {
        self.my_token_balance
            .handle_token_event(TokenEvent::HotOrNotOutcomePayout {
                amount: earnings_amount as u64,
                details: hot_or_not_outcome_details,
                timestamp,
            });
    }

    fn validate_incoming_bet(
        &self,
        token: &dyn TokenTransactions,
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

        let utlility_token_balance = token.get_current_token_balance();

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
}

#[derive(Deserialize, Serialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, Post>,
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
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
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
    pub device_identities: Vec<DeviceIdentity>,
    #[serde(default)]
    pub ml_feed_cache: Vec<MLFeedCacheItem>,
    #[serde(default)]
    pub cdao_canisters: Vec<DeployedCdaoCanisters>,
    // list of root token canisters
    #[serde(skip, default = "_default_token_list")]
    pub token_roots: ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory>,
    #[serde(default)]
    pub ml_data: MLData,
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

    fn get_post(&self, post_id: u64) -> Result<Post, String> {
        let post = self.all_created_posts.get(&post_id).unwrap();
        if post.status.eq(&PostStatus::Deleted) {
            return Err("Post not found".to_owned());
        }
        Ok(post.clone())
    }

    pub(crate) fn get_post_for_frontend(
        &self,
        post_id: u64,
        caller: Principal,
    ) -> PostDetailsForFrontend {
        let post = self.get_post(post_id).unwrap();
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
            posts_index_sorted_by_home_feed_score: PostScoreIndex::default(),
            posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex::default(),
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
            device_identities: Vec::new(),
            ml_feed_cache: Vec::new(),
            cdao_canisters: Vec::new(),
            token_roots: _default_token_list(),
            ml_data: MLData::default(),
            empty_canisters: AllotedEmptyCanister::default(),
        }
    }
}
