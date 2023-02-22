use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use speedy::{Readable, Writable};

#[derive(Readable, Writable, Serialize, Deserialize, CandidType, Clone, Default, Debug)]
pub enum PostStatus {
    #[default]
    Uploaded,
    Transcoding,
    CheckingExplicitness,
    BannedForExplicitness,
    ReadyToView,
    BannedDueToUserReporting,
    Deleted,
}

#[derive(Serialize, CandidType, Deserialize, Debug)]
pub struct PostDetailsForFrontend {
    pub id: u64,
    pub created_by_display_name: Option<String>,
    pub created_by_unique_user_name: Option<String>,
    pub created_by_user_principal_id: Principal,
    pub created_by_profile_photo_url: Option<String>,
    pub created_at: SystemTime,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub total_view_count: u64,
    pub like_count: u64,
    pub liked_by_me: bool,
    pub home_feed_ranking_score: u64,
    pub hot_or_not_feed_ranking_score: Option<u64>,
}
