use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Serialize, CandidType, Deserialize)]
pub struct PostDetailsFromFrontend {
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
}
