use std::time::{Duration, SystemTime};

use candid::Principal;

pub mod post_score_home_index;
pub mod post_score_hot_or_not_index;
pub mod post_score_index;
pub mod post_score_index_item;

pub type PublisherCanisterId = Principal;
pub type PostId = u64;
pub type Score = u64;
pub type CreatedAt = SystemTime;
pub type GlobalPostId = (PublisherCanisterId, PostId);

// Latest posts within 48 hrs
pub const LATEST_POSTS_WINDOW: Duration = Duration::from_secs(48 * 60 * 60);
