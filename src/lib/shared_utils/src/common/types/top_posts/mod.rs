use std::time::SystemTime;

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
