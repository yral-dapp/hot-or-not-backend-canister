use std::collections::{BTreeMap, BTreeSet};

use candid::{Deserialize, Principal};
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        hot_or_not::PlacedBetDetail, post::Post, profile::UserProfile, token::TokenBalance,
    },
    common::types::{
        known_principal::KnownPrincipalMap, top_posts::post_score_index::PostScoreIndex,
    },
};

use self::version_details::VersionDetails;

pub mod version_details;

#[derive(Default, Deserialize, Serialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, Post>,
    #[serde(default)]
    pub all_hot_or_not_bets_placed: BTreeSet<PlacedBetDetail>,
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,
    // TODO: remove this index
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    // TODO: remove this index
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
}
