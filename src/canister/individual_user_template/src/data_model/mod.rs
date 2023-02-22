use std::collections::{BTreeMap, BTreeSet};

use candid::{CandidType, Deserialize, Principal};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        post::Post, profile::UserProfile, token::TokenBalance,
    },
    common::types::{
        known_principal::KnownPrincipalMap, top_posts::post_score_index::v0::PostScoreIndex,
    },
};

use self::version_details::VersionDetails;

pub mod hot_or_not;
pub mod version_details;

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, Post>,
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
}
