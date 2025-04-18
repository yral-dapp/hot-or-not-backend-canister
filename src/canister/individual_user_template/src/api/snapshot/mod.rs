use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    time::SystemTime,
};

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use serde::Serialize;
use serde_json_any_key::*;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        follow::{FollowEntryDetail, FollowEntryId},
        hot_or_not::{
            AggregateStats, BetDetails, GlobalBetId, GlobalRoomId, HotOrNotDetails,
            PlacedBetDetail, RoomDetailsV1, SlotDetailsV1, SlotId, StablePrincipal,
        },
        migration::MigrationInfo,
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        session::SessionType,
        token::TokenBalance,
    },
    common::types::{
        app_primitive_type::PostId,
        known_principal::KnownPrincipalMap,
        top_posts::{
            post_score_index_item::{PostScoreIndexItem, PostStatus},
            PublisherCanisterId, Score,
        },
        utility_token::token_event::TokenEvent,
        version_details::VersionDetails,
    },
};

use crate::data_model::_default_room_details;
use crate::data_model::{
    CanisterData, _default_bet_details, _default_post_principal_map, _default_slot_details_map,
};

pub mod get_snapshot;
pub mod serde_json_snapshot_test;

#[derive(Deserialize, Serialize)]
pub struct CanisterDataForSnapshot {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, PostForSnapshot>,
    #[serde(with = "any_key_map")]
    pub room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1>,
    #[serde(with = "any_key_map")]
    pub bet_details_map: BTreeMap<GlobalBetId, BetDetails>,
    #[serde(with = "any_key_map")]
    pub post_principal_map: BTreeMap<(PostId, StablePrincipal), ()>,
    #[serde(with = "any_key_map")]
    pub slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1>,
    #[serde(with = "any_key_map")]
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
    #[serde(with = "any_key_map")]
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalanceForSnapshot,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    pub session_type: Option<SessionType>,
    pub last_access_time: Option<SystemTime>,
    pub migration_info: MigrationInfo,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct PostForSnapshot {
    pub id: u64,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub created_at: SystemTime,
    pub likes: HashSet<Principal>,
    pub share_count: u64,
    pub view_stats: PostViewStatistics,
    pub home_feed_score: FeedScore,
    pub hot_or_not_details: Option<HotOrNotDetailsForSnapshot>,
    #[serde(default)]
    pub is_nsfw: bool,
    pub slots_left_to_be_computed: HashSet<SlotId>,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct HotOrNotDetailsForSnapshot {
    pub hot_or_not_feed_score: FeedScore,
    pub aggregate_stats: AggregateStats,
}

#[derive(Default, Clone, Deserialize, CandidType, Debug, Serialize)]
pub struct TokenBalanceForSnapshot {
    pub utility_token_balance: u64,
    #[serde(with = "any_key_map")]
    pub utility_token_transaction_history: BTreeMap<u64, TokenEvent>,
    pub lifetime_earnings: u64,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct FollowDataForSnapshot {
    pub follower: FollowListForSnapshot,
    pub following: FollowListForSnapshot,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct FollowListForSnapshot {
    #[serde(with = "any_key_map")]
    pub sorted_index: BTreeMap<FollowEntryId, FollowEntryDetail>,
    #[serde(with = "any_key_map")]
    pub members: HashMap<FollowEntryDetail, FollowEntryId>,
}

#[derive(Default, Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct PostScoreIndexForSnapshot {
    #[serde(with = "any_key_map")]
    pub items_sorted_by_score: BTreeMap<Score, Vec<PostScoreIndexItem>>,
    #[serde(with = "any_key_map")]
    pub item_presence_index: HashMap<(PublisherCanisterId, PostId), Score>,
}

impl From<&CanisterData> for CanisterDataForSnapshot {
    fn from(canister_data: &CanisterData) -> Self {
        let mut all_created_posts: BTreeMap<u64, PostForSnapshot> = BTreeMap::new();
        canister_data
            .get_all_posts_cloned()
            .into_iter()
            .for_each(|(k, v)| {
                let hot_or_not_details = v.hot_or_not_details.clone();
                let hot_or_not_details_snapshot =
                    hot_or_not_details.map(|hot_or_not_details| HotOrNotDetailsForSnapshot {
                        hot_or_not_feed_score: hot_or_not_details.hot_or_not_feed_score,
                        aggregate_stats: hot_or_not_details.aggregate_stats.clone(),
                    });

                let post_details = PostForSnapshot {
                    id: v.id,
                    description: v.description.clone(),
                    hashtags: v.hashtags.clone(),
                    video_uid: v.video_uid.clone(),
                    status: v.status,
                    created_at: v.created_at,
                    likes: v.likes.clone(),
                    share_count: v.share_count,
                    view_stats: v.view_stats.clone(),
                    home_feed_score: v.home_feed_score.clone(),
                    hot_or_not_details: hot_or_not_details_snapshot,
                    is_nsfw: v.is_nsfw,
                    slots_left_to_be_computed: v.slots_left_to_be_computed.clone(),
                };

                all_created_posts.insert(k, post_details);
            });

        let mut room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1> = BTreeMap::new();
        canister_data.room_details_map.iter().for_each(|(k, v)| {
            room_details_map.insert(k, v.clone());
        });

        let mut bet_details_map: BTreeMap<GlobalBetId, BetDetails> = BTreeMap::new();
        canister_data.bet_details_map.iter().for_each(|(k, v)| {
            bet_details_map.insert(k, v.clone());
        });

        let mut post_principal_map: BTreeMap<(PostId, StablePrincipal), ()> = BTreeMap::new();
        canister_data.post_principal_map.iter().for_each(|(k, v)| {
            post_principal_map.insert(k, v.clone());
        });

        let mut slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1> = BTreeMap::new();
        canister_data.slot_details_map.iter().for_each(|(k, v)| {
            slot_details_map.insert(k, v.clone());
        });

        let my_token_balance = TokenBalanceForSnapshot {
            utility_token_balance: canister_data.my_token_balance.utility_token_balance,
            utility_token_transaction_history: canister_data
                .my_token_balance
                .utility_token_transaction_history
                .clone(),
            lifetime_earnings: canister_data.my_token_balance.lifetime_earnings,
        };

        Self {
            all_created_posts,
            room_details_map,
            bet_details_map,
            post_principal_map,
            slot_details_map,
            all_hot_or_not_bets_placed: canister_data.all_hot_or_not_bets_placed.clone(),
            known_principal_ids: canister_data.known_principal_ids.clone(),
            my_token_balance,
            profile: canister_data.profile.clone(),
            version_details: canister_data.version_details.clone(),
            session_type: canister_data.session_type,
            last_access_time: canister_data.last_access_time,
            migration_info: canister_data.migration_info,
        }
    }
}

impl From<CanisterDataForSnapshot> for CanisterData {
    fn from(canister_data_for_snapshot: CanisterDataForSnapshot) -> Self {
        let mut all_created_posts: BTreeMap<u64, Post> = BTreeMap::new();
        canister_data_for_snapshot
            .all_created_posts
            .iter()
            .for_each(|(k, v)| {
                let hot_or_not_details_snapshot = v.hot_or_not_details.clone();
                let hot_or_not_details =
                    hot_or_not_details_snapshot.map(|hot_or_not_details_snapshot| {
                        HotOrNotDetails {
                            hot_or_not_feed_score: hot_or_not_details_snapshot
                                .hot_or_not_feed_score,
                            aggregate_stats: hot_or_not_details_snapshot.aggregate_stats.clone(),
                            slot_history: BTreeMap::new(),
                        }
                    });

                let post_details = Post {
                    id: v.id,
                    description: v.description.clone(),
                    hashtags: v.hashtags.clone(),
                    video_uid: v.video_uid.clone(),
                    status: v.status,
                    created_at: v.created_at,
                    likes: v.likes.clone(),
                    share_count: v.share_count,
                    view_stats: v.view_stats.clone(),
                    home_feed_score: v.home_feed_score.clone(),
                    hot_or_not_details,
                    is_nsfw: v.is_nsfw,
                    slots_left_to_be_computed: v.slots_left_to_be_computed.clone(),
                };

                all_created_posts.insert(k.clone(), post_details);
            });

        let mut room_details_map = _default_room_details();
        canister_data_for_snapshot
            .room_details_map
            .iter()
            .for_each(|(k, v)| {
                room_details_map.insert(*k, v.clone());
            });

        let mut bet_details_map = _default_bet_details();
        canister_data_for_snapshot
            .bet_details_map
            .iter()
            .for_each(|(k, v)| {
                bet_details_map.insert(k.clone(), v.clone());
            });

        let mut post_principal_map = _default_post_principal_map();
        canister_data_for_snapshot
            .post_principal_map
            .iter()
            .for_each(|(k, v)| {
                post_principal_map.insert(k.clone(), v.clone());
            });

        let mut slot_details_map = _default_slot_details_map();
        canister_data_for_snapshot
            .slot_details_map
            .iter()
            .for_each(|(k, v)| {
                slot_details_map.insert(*k, v.clone());
            });

        let my_token_balance = TokenBalance {
            utility_token_balance: canister_data_for_snapshot
                .my_token_balance
                .utility_token_balance,
            utility_token_transaction_history: canister_data_for_snapshot
                .my_token_balance
                .utility_token_transaction_history
                .clone(),
            lifetime_earnings: canister_data_for_snapshot
                .my_token_balance
                .lifetime_earnings,
        };

        let mut canister_data = CanisterData::default();

        canister_data.room_details_map = room_details_map;
        canister_data.bet_details_map = bet_details_map;
        canister_data.post_principal_map = post_principal_map;
        canister_data.slot_details_map = slot_details_map;
        canister_data.all_hot_or_not_bets_placed =
            canister_data_for_snapshot.all_hot_or_not_bets_placed;
        canister_data.known_principal_ids = canister_data_for_snapshot.known_principal_ids;
        canister_data.my_token_balance = my_token_balance;
        canister_data.profile = canister_data_for_snapshot.profile;
        canister_data.version_details = canister_data_for_snapshot.version_details;
        canister_data.session_type = canister_data_for_snapshot.session_type;
        canister_data.last_access_time = canister_data_for_snapshot.last_access_time;
        canister_data.migration_info = canister_data_for_snapshot.migration_info;

        canister_data.set_all_created_posts(all_created_posts);

        canister_data
    }
}
