use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashSet},
    time::SystemTime,
};

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use serde::Serialize;
use serde_json_any_key::*;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        configuration::IndividualUserConfiguration,
        follow::FollowData,
        hot_or_not::{
            AggregateStats, BetDetails, BetMaker, BetMakerPrincipal, GlobalBetId, GlobalRoomId,
            HotOrNotDetails, PlacedBetDetail, RoomDetailsV1, RoomId, SlotDetailsV1, SlotId,
            StablePrincipal,
        },
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        token::TokenBalance,
    },
    common::types::{
        app_primitive_type::PostId,
        known_principal::KnownPrincipalMap,
        top_posts::{post_score_index::PostScoreIndex, post_score_index_item::PostStatus},
        version_details::VersionDetails,
    },
};

use self::memory::{
    get_bet_details_memory, get_post_principal_memory, get_room_details_memory,
    get_slot_details_memory, Memory,
};

pub mod memory;

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
        }
    }
}

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
    pub configuration: IndividualUserConfiguration,
    pub follow_data: FollowData,
    #[serde(with = "any_key_map")]
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
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
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub hot_or_not_details: Option<HotOrNotDetailsForSnapshot>,
    #[serde(default)]
    pub is_nsfw: bool,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct HotOrNotDetailsForSnapshot {
    pub hot_or_not_feed_score: FeedScore,
    pub aggregate_stats: AggregateStats,
}

impl From<&CanisterData> for CanisterDataForSnapshot {
    fn from(canister_data: &CanisterData) -> Self {
        let mut all_created_posts: BTreeMap<u64, PostForSnapshot> = BTreeMap::new();
        canister_data.all_created_posts.iter().for_each(|(k, v)| {
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
                creator_consent_for_inclusion_in_hot_or_not: v
                    .creator_consent_for_inclusion_in_hot_or_not,
                hot_or_not_details: hot_or_not_details_snapshot,
                is_nsfw: v.is_nsfw,
            };

            all_created_posts.insert(k.clone(), post_details);
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

        Self {
            all_created_posts,
            room_details_map,
            bet_details_map,
            post_principal_map,
            slot_details_map,
            all_hot_or_not_bets_placed: canister_data.all_hot_or_not_bets_placed.clone(),
            configuration: canister_data.configuration.clone(),
            follow_data: canister_data.follow_data.clone(),
            known_principal_ids: canister_data.known_principal_ids.clone(),
            my_token_balance: canister_data.my_token_balance.clone(),
            posts_index_sorted_by_home_feed_score: canister_data
                .posts_index_sorted_by_home_feed_score
                .clone(),
            posts_index_sorted_by_hot_or_not_feed_score: canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .clone(),
            principals_i_follow: canister_data.principals_i_follow.clone(),
            principals_that_follow_me: canister_data.principals_that_follow_me.clone(),
            profile: canister_data.profile.clone(),
            version_details: canister_data.version_details.clone(),
        }
    }
}

impl From<CanisterDataForSnapshot> for CanisterData {
    fn from(canister_data: CanisterDataForSnapshot) -> Self {
        let mut all_created_posts: BTreeMap<u64, Post> = BTreeMap::new();
        canister_data.all_created_posts.iter().for_each(|(k, v)| {
            let hot_or_not_details_snapshot = v.hot_or_not_details.clone();
            let hot_or_not_details =
                hot_or_not_details_snapshot.map(|hot_or_not_details_snapshot| HotOrNotDetails {
                    hot_or_not_feed_score: hot_or_not_details_snapshot.hot_or_not_feed_score,
                    aggregate_stats: hot_or_not_details_snapshot.aggregate_stats.clone(),
                    slot_history: BTreeMap::new(),
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
                creator_consent_for_inclusion_in_hot_or_not: v
                    .creator_consent_for_inclusion_in_hot_or_not,
                hot_or_not_details: hot_or_not_details,
                is_nsfw: v.is_nsfw,
            };

            all_created_posts.insert(k.clone(), post_details);
        });

        let mut room_details_map = _default_room_details();
        canister_data.room_details_map.iter().for_each(|(k, v)| {
            room_details_map.insert(*k, v.clone());
        });

        let mut bet_details_map = _default_bet_details();
        canister_data.bet_details_map.iter().for_each(|(k, v)| {
            bet_details_map.insert(k.clone(), v.clone());
        });

        let mut post_principal_map = _default_post_principal_map();
        canister_data.post_principal_map.iter().for_each(|(k, v)| {
            post_principal_map.insert(k.clone(), v.clone());
        });

        let mut slot_details_map = _default_slot_details_map();
        canister_data.slot_details_map.iter().for_each(|(k, v)| {
            slot_details_map.insert(*k, v.clone());
        });

        Self {
            all_created_posts,
            room_details_map,
            bet_details_map,
            post_principal_map,
            slot_details_map,
            all_hot_or_not_bets_placed: canister_data.all_hot_or_not_bets_placed,
            configuration: canister_data.configuration,
            follow_data: canister_data.follow_data,
            known_principal_ids: canister_data.known_principal_ids,
            my_token_balance: canister_data.my_token_balance,
            posts_index_sorted_by_home_feed_score: canister_data
                .posts_index_sorted_by_home_feed_score,
            posts_index_sorted_by_hot_or_not_feed_score: canister_data
                .posts_index_sorted_by_hot_or_not_feed_score,
            principals_i_follow: canister_data.principals_i_follow,
            principals_that_follow_me: canister_data.principals_that_follow_me,
            profile: canister_data.profile,
            version_details: canister_data.version_details,
        }
    }
}
