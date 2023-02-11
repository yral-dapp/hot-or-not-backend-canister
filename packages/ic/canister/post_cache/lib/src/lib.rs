use candid::{CandidType, Deserialize};
use ic_stable_memory::{collections::hash_map::SHashMap, utils::ic_types::SPrincipal};
use shared_utils::{
    access_control::UserAccessRole,
    common::types::known_principal::KnownPrincipalMap,
    types::top_posts::{v0::PostScoreIndexItem, v1::PostScoreIndex},
};
use std::collections::{BTreeSet, HashMap};

pub mod access_control;
pub mod model;
pub mod post;
pub mod score_ranking;

// * Stable Variables
pub type MyKnownPrincipalIdsMap = KnownPrincipalMap;

// * Stable collections
pub type AccessControlMap = SHashMap<SPrincipal, Vec<UserAccessRole>>;
pub type PostsIndexSortedByScore = BTreeSet<PostScoreIndexItem>;
pub type PostsIndexSortedByHomeFeedScore = PostScoreIndex;
pub type PostsIndexSortedByHotOrNotFeedScore = PostScoreIndex;

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    pub my_known_principal_ids_map: KnownPrincipalMap,
    pub access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>>,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
}
