use std::collections::{BTreeMap, BTreeSet};

use candid::{CandidType, Deserialize, Principal};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        post::v1::Post as PostV1, token::TokenBalance,
    },
    common::types::known_principal::KnownPrincipalMapV1,
    types::top_posts::v1::PostScoreIndex,
};

use self::{profile::v1::UserProfile as UserProfileV1, version_details::VersionDetails};

pub mod hot_or_not;
pub mod profile;
pub mod version_details;

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, PostV1>,
    pub known_principal_ids: KnownPrincipalMapV1,
    pub my_token_balance: TokenBalance,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfileV1,
    pub version_details: VersionDetails,
}
