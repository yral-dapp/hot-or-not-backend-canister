use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use shared_utils::{
    access_control::UserAccessRole,
    common::types::{
        known_principal::KnownPrincipalMap, top_posts::post_score_index::PostScoreIndex,
    },
};
use std::collections::HashMap;

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterData {
    pub known_principal_ids: KnownPrincipalMap,
    // TODO: remove this field
    pub access_control_map: HashMap<Principal, Vec<UserAccessRole>>,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
}
