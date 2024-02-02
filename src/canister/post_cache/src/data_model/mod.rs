use candid::{CandidType, Deserialize};
use serde::Serialize;
use shared_utils::common::types::{
    known_principal::KnownPrincipalMap,
    top_posts::{
        post_score_home_index::PostScoreHomeIndex,
        post_score_hot_or_not_index::PostScoreHotOrNotIndex, post_score_index::PostScoreIndex,
    },
    version_details::VersionDetails,
};

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterData {
    pub known_principal_ids: KnownPrincipalMap,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    #[serde(default)]
    pub posts_index_sorted_by_home_feed_score_v1: PostScoreHomeIndex,
    #[serde(default)]
    pub posts_index_sorted_by_hot_or_not_feed_score_v1: PostScoreHotOrNotIndex,
    #[serde(default)]
    pub version_details: Option<VersionDetails>,
}
