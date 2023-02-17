use candid::{CandidType, Deserialize, Principal};
use ic_stable_memory::utils::ic_types::SPrincipal;
use serde::Serialize;
use shared_utils::{
    access_control::UserAccessRole,
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalMapV1},
    types::top_posts::post_score_index::{
        v0::PostScoreIndex, v1::PostScoreIndex as PostScoreIndexV1,
    },
};
use std::collections::HashMap;

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    pub my_known_principal_ids_map: KnownPrincipalMap,
    pub access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>>,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
}

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterDataV2 {
    pub my_known_principal_ids_map: KnownPrincipalMapV1,
    pub access_control_map: HashMap<Principal, Vec<UserAccessRole>>,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndexV1,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndexV1,
}

impl From<CanisterData> for CanisterDataV2 {
    fn from(canister_data: CanisterData) -> Self {
        let CanisterData {
            my_known_principal_ids_map,
            access_control_map,
            posts_index_sorted_by_home_feed_score,
            posts_index_sorted_by_hot_or_not_feed_score,
        } = canister_data;

        let my_known_principal_ids_map = my_known_principal_ids_map
            .iter()
            .map(|(k, v)| (k.clone(), v.0))
            .collect();

        let access_control_map = access_control_map
            .into_iter()
            .map(|(principal, roles)| (principal.0, roles))
            .collect();

        let posts_index_sorted_by_home_feed_score = posts_index_sorted_by_home_feed_score.into();

        let posts_index_sorted_by_hot_or_not_feed_score =
            posts_index_sorted_by_hot_or_not_feed_score.into();

        Self {
            my_known_principal_ids_map,
            access_control_map,
            posts_index_sorted_by_home_feed_score,
            posts_index_sorted_by_hot_or_not_feed_score,
        }
    }
}
