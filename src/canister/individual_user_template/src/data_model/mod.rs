use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
};

use candid::{Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use serde::Serialize;
use serde_json_any_key::*;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        configuration::IndividualUserConfiguration,
        follow::FollowData,
        hot_or_not::{
            BetDetails, BetMaker, BetMakerPrincipal, GlobalBetId, GlobalRoomId, PlacedBetDetail,
            RoomDetailsV1, RoomId, SlotDetailsV1, SlotId, StablePrincipal,
        },
        post::Post,
        profile::UserProfile,
        token::TokenBalance,
    },
    common::types::{
        app_primitive_type::PostId, known_principal::KnownPrincipalMap,
        top_posts::post_score_index::PostScoreIndex, version_details::VersionDetails,
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

fn _default_room_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory())
}

fn _default_bet_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory())
}

fn _default_post_principal_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory())
}

fn _default_slot_details_map(
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
    pub all_created_posts: BTreeMap<u64, Post>,
    #[serde(with = "any_key_map")]
    pub room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1>,
    #[serde(with = "any_key_map")]
    pub bet_details_map: BTreeMap<GlobalBetId, BetDetails>,
    #[serde(with = "any_key_map")]
    pub post_principal_map: BTreeMap<(PostId, StablePrincipal), ()>,
    #[serde(with = "any_key_map")]
    pub slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1>,
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

impl From<&CanisterData> for CanisterDataForSnapshot {
    fn from(canister_data: &CanisterData) -> Self {
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
            all_created_posts: canister_data.all_created_posts.clone(),
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
            all_created_posts: canister_data.all_created_posts,
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
