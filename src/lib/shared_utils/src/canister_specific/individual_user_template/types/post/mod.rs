use candid::{CandidType, Deserialize, Principal};
use ic_stable_memory::utils::ic_types::SPrincipal;
use serde::{Deserializer, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, SystemTime},
};

use crate::{
    canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend,
    types::{
        canister_specific::individual_user_template::post::{PostDetailsForFrontend, PostStatus},
        post::PostDetailsFromFrontend,
    },
};

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct Post {
    pub id: u64,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub status: PostStatus,
    pub created_at: SystemTime,
    #[serde(deserialize_with = "principal_deserializer")]
    pub likes: HashSet<Principal>,
    pub share_count: u64,
    pub view_stats: PostViewStatistics,
    pub homefeed_ranking_score: u64,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub hot_or_not_feed_details: Option<HotOrNotFeedDetails>,
}

fn principal_deserializer<'de, D>(deserializer: D) -> Result<HashSet<Principal>, D::Error>
where
    D: Deserializer<'de>,
{
    let previous: HashSet<SPrincipal> = HashSet::deserialize(deserializer)?;

    Ok(previous.into_iter().map(|principal| principal.0).collect())
}

#[derive(Deserialize, CandidType)]
pub enum PostViewDetailsFromFrontend {
    WatchedPartially {
        percentage_watched: u8,
    },
    WatchedMultipleTimes {
        watch_count: u8,
        percentage_watched: u8,
    },
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct PostViewStatistics {
    pub total_view_count: u64,
    pub threshold_view_count: u64,
    pub average_watch_percentage: u8,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct HotOrNotFeedDetails {
    pub score: u64,
    #[serde(deserialize_with = "principal_deserializer")]
    pub upvotes: HashSet<Principal>,
    #[serde(deserialize_with = "principal_deserializer")]
    pub downvotes: HashSet<Principal>,
    // TODO: consider video age, remove after 48 hours
}

impl Post {
    pub fn new(
        id: u64,
        post_details_from_frontend: PostDetailsFromFrontend,
        time_provider: &impl Fn() -> SystemTime,
    ) -> Self {
        let mut post = Post {
            id,
            description: post_details_from_frontend.description,
            hashtags: post_details_from_frontend.hashtags,
            video_uid: post_details_from_frontend.video_uid,
            status: PostStatus::Uploaded,
            created_at: time_provider(),
            likes: HashSet::new(),
            share_count: 0,
            view_stats: PostViewStatistics {
                total_view_count: 1, // To not have divide by zero errors
                threshold_view_count: 0,
                average_watch_percentage: 0,
            },
            homefeed_ranking_score: 0,
            creator_consent_for_inclusion_in_hot_or_not: post_details_from_frontend
                .creator_consent_for_inclusion_in_hot_or_not,
            hot_or_not_feed_details: None,
        };

        if post.creator_consent_for_inclusion_in_hot_or_not {
            post.hot_or_not_feed_details = Some(HotOrNotFeedDetails {
                score: 0,
                upvotes: HashSet::new(),
                downvotes: HashSet::new(),
            });
        }

        post
    }

    pub fn update_status(&mut self, status: PostStatus) {
        self.status = status;
    }

    pub fn toggle_like_status(
        &mut self,
        user_principal_id: &Principal,
        time_provider: &impl Fn() -> SystemTime,
    ) -> bool {
        // if liked, return true & if unliked, return false
        if self.likes.contains(user_principal_id) {
            self.likes.remove(user_principal_id);

            self.recalculate_home_feed_score(time_provider);

            return false;
        } else {
            self.likes.insert(user_principal_id.clone());

            self.recalculate_home_feed_score(time_provider);

            return true;
        }
    }

    pub fn increment_share_count(&mut self, time_provider: &impl Fn() -> SystemTime) -> u64 {
        self.share_count += 1;
        self.recalculate_home_feed_score(time_provider);
        self.share_count
    }

    fn recalculate_average_watched(&self, percentage_watched: u8, additional_views: u8) -> u8 {
        (((self.view_stats.average_watch_percentage as u64 * self.view_stats.total_view_count)
            + (100 * (additional_views - 1)) as u64
            + percentage_watched as u64)
            / (self.view_stats.total_view_count + additional_views as u64)) as u8
    }

    pub fn recalculate_home_feed_score(&mut self, time_provider: &impl Fn() -> SystemTime) {
        let likes_component = match self.view_stats.total_view_count {
            0 => 0,
            _ => 1000 * self.likes.len() as u64 * 10 / self.view_stats.total_view_count,
        };
        let threshold_views_component = match self.view_stats.total_view_count {
            0 => 0,
            _ => 1000 * self.view_stats.threshold_view_count / self.view_stats.total_view_count,
        };
        let average_percent_viewed_component =
            1000 * self.view_stats.average_watch_percentage as u64;
        let post_share_component = match self.view_stats.total_view_count {
            0 => 0,
            _ => 1000 * self.share_count * 100 / self.view_stats.total_view_count,
        };

        let current_time = time_provider();
        let subtracting_factor = (current_time
            .duration_since(self.created_at)
            .unwrap_or(Duration::ZERO)
            .as_secs())
            / (60 * 60 * 4);
        let age_of_video_component = (1000 - 50 * subtracting_factor).max(0);

        self.homefeed_ranking_score = likes_component
            + threshold_views_component
            + average_percent_viewed_component
            + post_share_component
            + age_of_video_component;

        // * update score index for top posts of this user
        // TODO: these index scores need to be recalculated on every update
        // score_ranking::update_post_home_feed_score_index_on_home_feed_post_score_recalculation(
        //     self.id,
        //     self.homefeed_ranking_score,
        // );
    }

    pub fn recalculate_hot_or_not_feed_score(&mut self, time_provider: &impl Fn() -> SystemTime) {
        if self.hot_or_not_feed_details.is_some() {
            let likes_component = match self.view_stats.total_view_count {
                0 => 0,
                _ => 1000 * self.likes.len() as u64 * 10 / self.view_stats.total_view_count,
            };

            let absolute_calc_for_hots_ratio =
                (((((self.hot_or_not_feed_details.as_ref().unwrap().upvotes.len() as u64)
                    / (self.hot_or_not_feed_details.as_ref().unwrap().upvotes.len() as u64
                        + self
                            .hot_or_not_feed_details
                            .as_ref()
                            .unwrap()
                            .downvotes
                            .len() as u64
                        + 1))
                    * 1000)
                    - 500) as i64)
                    .abs();
            let hots_ratio_component = 1000 * (1000 - (absolute_calc_for_hots_ratio as u64 * 2));
            let threshold_views_component =
                1000 * self.view_stats.threshold_view_count / self.view_stats.total_view_count;
            let average_percent_viewed_component =
                1000 * self.view_stats.average_watch_percentage as u64;
            let post_share_component =
                1000 * self.share_count * 100 / self.view_stats.total_view_count;
            let hot_or_not_participation_component = 1000
                * ((self.hot_or_not_feed_details.as_ref().unwrap().upvotes.len() as u64
                    + self
                        .hot_or_not_feed_details
                        .as_ref()
                        .unwrap()
                        .downvotes
                        .len() as u64)
                    / self.view_stats.total_view_count);

            let current_time = time_provider();
            let subtracting_factor = (current_time
                .duration_since(self.created_at)
                .unwrap_or(Duration::ZERO)
                .as_secs())
                / (60 * 60 * 4);
            let age_of_video_component = (1000 - 50 * subtracting_factor).max(0);

            self.hot_or_not_feed_details.as_mut().unwrap().score = likes_component
                + hots_ratio_component
                + threshold_views_component
                + average_percent_viewed_component
                + post_share_component
                + hot_or_not_participation_component
                + age_of_video_component;

            // * update score index for top posts of this user
            // TODO: needs an alternative
            // score_ranking::update_post_score_index_on_hot_or_not_feed_post_score_recalculation(
            //     self.id,
            //     self.hot_or_not_feed_details.as_ref().unwrap().score,
            // );
        }
    }

    pub fn add_view_details(
        &mut self,
        details: PostViewDetailsFromFrontend,
        time_provider: &impl Fn() -> SystemTime,
    ) {
        match details {
            PostViewDetailsFromFrontend::WatchedPartially { percentage_watched } => {
                assert!(percentage_watched <= 100 && percentage_watched > 0);
                self.view_stats.average_watch_percentage =
                    self.recalculate_average_watched(percentage_watched, 1);
                self.view_stats.total_view_count += 1;
                if percentage_watched > 20 {
                    self.view_stats.threshold_view_count += 1;
                }
            }
            PostViewDetailsFromFrontend::WatchedMultipleTimes {
                watch_count,
                percentage_watched,
            } => {
                assert!(percentage_watched <= 100 && percentage_watched > 0);
                self.view_stats.average_watch_percentage =
                    self.recalculate_average_watched(percentage_watched, watch_count);
                self.view_stats.total_view_count += watch_count as u64;
                if watch_count > 1 {
                    self.view_stats.threshold_view_count += (watch_count - 1) as u64;
                }
                if percentage_watched > 20 {
                    self.view_stats.threshold_view_count += 1;
                }
            }
        }

        self.recalculate_home_feed_score(time_provider);
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
            status: self.status.clone(),
            total_view_count: self.view_stats.total_view_count,
            like_count: self.likes.len() as u64,
            liked_by_me: self.likes.contains(&caller),
            home_feed_ranking_score: self.homefeed_ranking_score,
            hot_or_not_feed_ranking_score: self
                .hot_or_not_feed_details
                .as_ref()
                .map(|details| details.score),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn when_new_post_created_then_their_hot_or_not_feed_score_is_calculated() {
        let post = Post::new(
            0,
            PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &|| SystemTime::now(),
        );

        assert_eq!(post.hot_or_not_feed_details.unwrap().score, 0);
    }
}
