use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::SystemTime,
};

use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use serde::Serialize;
use serde_json_any_key::*;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters,
        cents::CentsToken,
        follow::{FollowEntryDetail, FollowEntryId},
        hot_or_not::{
            AggregateStats, BetDetails, GlobalBetId, GlobalRoomId, PlacedBetDetail, RoomDetailsV1,
            SlotDetailsV1, SlotId, StablePrincipal,
        },
        migration::MigrationInfo,
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        pump_n_dump::ParticipatedGameInfo,
        session::SessionType,
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

use crate::data_model::pump_n_dump::TokenBetGame;
use crate::data_model::{
    CanisterData, _default_token_list,
    pump_n_dump::{NatStore, _default_lp},
};

pub mod get_snapshot;
pub mod serde_json_snapshot_test;

#[derive(Deserialize, Serialize)]
pub struct CanisterBackupSnapshot {
    pub canister_data_for_snapshot: CanisterDataForSnapshot,
    pub token_bet_game_for_snapshot: TokenBetGameForSnapshot,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CanisterDataForSnapshot {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, PostForSnapshot>,
    #[serde(with = "any_key_map")]
    pub known_principal_ids: KnownPrincipalMap,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    pub session_type: Option<SessionType>,
    pub last_access_time: Option<SystemTime>,
    pub migration_info: MigrationInfo,
    pub cdao_canisters: Vec<DeployedCdaoCanisters>,
    #[serde(with = "any_key_map")]
    pub token_roots: BTreeMap<Principal, ()>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenBetGameForSnapshot {
    pub referral_reward: Nat,
    pub onboarding_reward: Nat,
    pub games: Vec<ParticipatedGameInfo>,
    pub total_dumps: Nat,
    pub total_pumps: Nat,
    #[serde(with = "any_key_map")]
    pub liquidity_pools: BTreeMap<Principal, NatStore>,
    pub cents: CentsToken,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HotOrNotGameDetailsForSnapshot {
    #[serde(with = "any_key_map")]
    pub room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1>,
    #[serde(with = "any_key_map")]
    pub slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1>,
    #[serde(with = "any_key_map")]
    pub post_principal_map: BTreeMap<(PostId, StablePrincipal), ()>,
    #[serde(with = "any_key_map")]
    pub bet_details_map: BTreeMap<GlobalBetId, BetDetails>,
    #[serde(with = "any_key_map")]
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
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
    #[serde(default)]
    pub is_nsfw: bool,
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
                    is_nsfw: v.is_nsfw,
                };

                all_created_posts.insert(k, post_details);
            });

        let mut token_roots: BTreeMap<Principal, ()> = BTreeMap::new();
        canister_data.token_roots.iter().for_each(|(k, _)| {
            token_roots.insert(k, ());
        });

        Self {
            all_created_posts,
            known_principal_ids: canister_data.known_principal_ids.clone(),
            profile: canister_data.profile.clone(),
            version_details: canister_data.version_details.clone(),
            session_type: canister_data.session_type,
            last_access_time: canister_data.last_access_time,
            migration_info: canister_data.migration_info,
            cdao_canisters: canister_data.cdao_canisters.clone(),
            token_roots,
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
                    is_nsfw: v.is_nsfw,
                };

                all_created_posts.insert(*k, post_details);
            });

        let mut token_roots = _default_token_list();
        canister_data_for_snapshot
            .token_roots
            .iter()
            .for_each(|(k, _)| {
                token_roots.insert(*k, ());
            });

        let mut canister_data = CanisterData::default();

        canister_data.known_principal_ids = canister_data_for_snapshot.known_principal_ids;
        canister_data.profile = canister_data_for_snapshot.profile;
        canister_data.version_details = canister_data_for_snapshot.version_details;
        canister_data.session_type = canister_data_for_snapshot.session_type;
        canister_data.last_access_time = canister_data_for_snapshot.last_access_time;
        canister_data.migration_info = canister_data_for_snapshot.migration_info;
        canister_data.cdao_canisters = canister_data_for_snapshot.cdao_canisters;
        canister_data.token_roots = token_roots;

        canister_data.set_all_created_posts(all_created_posts);

        canister_data
    }
}

impl From<&TokenBetGame> for TokenBetGameForSnapshot {
    fn from(token_bet_game: &TokenBetGame) -> Self {
        let mut liquidity_pools: BTreeMap<Principal, NatStore> = BTreeMap::new();
        token_bet_game.liquidity_pools.iter().for_each(|(k, v)| {
            liquidity_pools.insert(k, v.clone());
        });

        Self {
            liquidity_pools,
            cents: token_bet_game.cents.clone(),
            referral_reward: token_bet_game.referral_reward.clone(),
            onboarding_reward: token_bet_game.onboarding_reward.clone(),
            games: token_bet_game.games.clone(),
            total_dumps: token_bet_game.total_dumps.clone(),
            total_pumps: token_bet_game.total_pumps.clone(),
        }
    }
}

impl From<TokenBetGameForSnapshot> for TokenBetGame {
    fn from(token_bet_game_for_snapshot: TokenBetGameForSnapshot) -> Self {
        let mut liquidity_pools = _default_lp();
        token_bet_game_for_snapshot
            .liquidity_pools
            .iter()
            .for_each(|(k, v)| {
                liquidity_pools.insert(*k, v.clone());
            });

        TokenBetGame {
            liquidity_pools,
            cents: token_bet_game_for_snapshot.cents.clone(),
            referral_reward: token_bet_game_for_snapshot.referral_reward.clone(),
            onboarding_reward: token_bet_game_for_snapshot.onboarding_reward.clone(),
            games: token_bet_game_for_snapshot.games.clone(),
            total_dumps: token_bet_game_for_snapshot.total_dumps.clone(),
            total_pumps: token_bet_game_for_snapshot.total_pumps.clone(),
        }
    }
}
