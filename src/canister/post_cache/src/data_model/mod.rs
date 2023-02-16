use candid::{CandidType, Deserialize};
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::{
    access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMap,
    types::top_posts::v1::PostScoreIndex,
};
use std::collections::HashMap;

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    pub my_known_principal_ids_map: KnownPrincipalMap,
    pub access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>>,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
}
