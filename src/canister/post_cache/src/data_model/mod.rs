use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use shared_utils::{
    access_control::UserAccessRole,
    common::types::{
        known_principal::KnownPrincipalMap, top_posts::post_score_index::v1::PostScoreIndex,
    },
};
use std::collections::HashMap;

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterData {
    pub my_known_principal_ids_map: KnownPrincipalMap,
    pub access_control_map: HashMap<Principal, Vec<UserAccessRole>>,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
}
