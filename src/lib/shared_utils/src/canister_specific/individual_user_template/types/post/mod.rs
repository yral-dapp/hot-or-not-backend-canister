use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::{collections::HashSet, time::SystemTime};

use crate::{
    canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend,
    common::types::top_posts::post_score_index_item::PostStatus,
};

use super::hot_or_not::BettingStatus;

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct Post {
    pub id: u64,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub created_at: SystemTime,
    pub likes: HashSet<Principal>,
    pub share_count: u64,
    pub view_stats: PostViewStatistics,
    #[serde(default)]
    pub is_nsfw: bool,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct FeedScore {
    pub current_score: u64,
    pub last_synchronized_score: u64,
    pub last_synchronized_at: SystemTime,
}

impl Default for FeedScore {
    fn default() -> Self {
        FeedScore {
            current_score: 0,
            last_synchronized_score: 0,
            last_synchronized_at: SystemTime::UNIX_EPOCH,
        }
    }
}

#[derive(Deserialize, CandidType)]
pub enum PostViewDetailsFromFrontend {
    WatchedPartially {
        percentage_watched: u8,
    },
    WatchedMultipleTimes {
        // * only send complete watches as part of this count
        watch_count: u8,
        percentage_watched: u8,
    },
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct PostViewStatistics {
    pub total_view_count: u64,
    pub threshold_view_count: u64,
    pub average_watch_percentage: u8,
}

#[derive(Serialize, CandidType, Deserialize, Debug, PartialEq, Eq)]
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
    pub hot_or_not_betting_status: Option<BettingStatus>,
    pub is_nsfw: bool,
}

#[derive(Serialize, CandidType, Deserialize)]
pub struct PostDetailsFromFrontend {
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub is_nsfw: bool,
}

impl From<Post> for PostDetailsFromFrontend {
    fn from(value: Post) -> Self {
        PostDetailsFromFrontend {
            description: value.description,
            hashtags: value.hashtags,
            video_uid: value.video_uid,
            creator_consent_for_inclusion_in_hot_or_not: true,
            is_nsfw: value.is_nsfw,
        }
    }
}

impl Post {
    pub fn add_view_details(&mut self, details: &PostViewDetailsFromFrontend) {
        match details {
            PostViewDetailsFromFrontend::WatchedPartially { percentage_watched } => {
                assert!(*percentage_watched <= 100 && *percentage_watched > 0);
                self.view_stats.average_watch_percentage =
                    self.recalculate_average_watched(*percentage_watched, 0);
                self.view_stats.total_view_count += 1;
                if *percentage_watched > 20 {
                    self.view_stats.threshold_view_count += 1;
                }
            }
            PostViewDetailsFromFrontend::WatchedMultipleTimes {
                watch_count,
                percentage_watched,
            } => {
                assert!(*percentage_watched <= 100 && *percentage_watched > 0);
                self.view_stats.average_watch_percentage =
                    self.recalculate_average_watched(*percentage_watched, *watch_count);
                self.view_stats.total_view_count += (*watch_count + 1) as u64;
                self.view_stats.threshold_view_count += *watch_count as u64;
                if *percentage_watched > 20 {
                    self.view_stats.threshold_view_count += 1;
                }
            }
        }
    }

    pub fn get_post_details_for_frontend_for_this_post(
        &self,
        user_profile: UserProfileDetailsForFrontend,
        caller: Principal,
    ) -> PostDetailsForFrontend {
        PostDetailsForFrontend {
            id: self.id,
            created_by_display_name: user_profile.display_name,
            created_by_unique_user_name: user_profile.unique_user_name,
            created_by_user_principal_id: user_profile.principal_id,
            created_by_profile_photo_url: user_profile.profile_picture_url,
            created_at: self.created_at,
            description: self.description.clone(),
            hashtags: self.hashtags.clone(),
            video_uid: self.video_uid.clone(),
            status: self.status,
            total_view_count: self.view_stats.total_view_count,
            like_count: self.likes.len() as u64,
            is_nsfw: self.is_nsfw,
            liked_by_me: self.likes.contains(&caller),
            home_feed_ranking_score: 0,
            hot_or_not_feed_ranking_score: None,
            hot_or_not_betting_status: None,
        }
    }

    pub fn increment_share_count(&mut self) -> u64 {
        self.share_count += 1;
        self.share_count
    }

    pub fn new(
        id: u64,
        post_details_from_frontend: &PostDetailsFromFrontend,
        current_time: &SystemTime,
    ) -> Self {
        Post {
            id,
            description: (*post_details_from_frontend.description).to_string(),
            hashtags: (*post_details_from_frontend.hashtags).to_vec(),
            video_uid: (*post_details_from_frontend.video_uid).to_string(),
            status: PostStatus::Uploaded,
            created_at: *current_time,
            likes: HashSet::new(),
            share_count: 0,
            is_nsfw: post_details_from_frontend.is_nsfw,
            view_stats: PostViewStatistics {
                total_view_count: 0,
                threshold_view_count: 0,
                average_watch_percentage: 0,
            },
        }
    }

    fn recalculate_average_watched(&self, percentage_watched: u8, full_view_count: u8) -> u8 {
        let earlier_sum_component =
            self.view_stats.average_watch_percentage as u64 * self.view_stats.total_view_count;
        let current_full_view_component = 100 * full_view_count as u64;
        let current_total_dividend =
            earlier_sum_component + current_full_view_component + percentage_watched as u64;
        let current_total_divisor = self.view_stats.total_view_count + full_view_count as u64 + 1;

        (current_total_dividend / current_total_divisor) as u8
    }

    pub fn toggle_like_status(&mut self, user_principal_id: &Principal) -> bool {
        // if liked, return true & if unliked, return false
        if self.likes.contains(user_principal_id) {
            self.likes.remove(user_principal_id);
            false
        } else {
            self.likes.insert(*user_principal_id);
            true
        }
    }

    pub fn update_status(&mut self, status: PostStatus) {
        self.status = status;
    }
}
