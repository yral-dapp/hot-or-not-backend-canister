// use crate::util::score_ranking;
// use ic_cdk::timer;
// use shared_utils::constant::{
//     SCORE_RECALCULATION_SYNC_INTERVAL_DURATION, TOP_POSTS_SYNC_INTERVAL_DURATION,
// };

// pub fn share_top_post_scores_with_post_cache_canister_v1() {
//     timer::set_timer_interval(
//         TOP_POSTS_SYNC_INTERVAL_DURATION,
//         score_ranking::send_top_home_feed_post_scores_to_post_cache_canister,
//     );
//     timer::set_timer_interval(
//         TOP_POSTS_SYNC_INTERVAL_DURATION,
//         score_ranking::send_top_hot_or_not_feed_post_scores_to_post_cache_canister,
//     );
// }

// pub fn update_post_scores_every_hour_v1() {
//     timer::set_timer_interval(
//         SCORE_RECALCULATION_SYNC_INTERVAL_DURATION,
//         score_ranking::update_home_feed_post_scores_for_every_post_in_posts_index_sorted_by_home_feed_score_v1,
//     );
//     timer::set_timer_interval(
//         SCORE_RECALCULATION_SYNC_INTERVAL_DURATION,
//         score_ranking::update_hot_or_not_feed_post_scores_for_every_post_in_posts_index_sorted_by_hot_or_not_feed_score_v1,
//     );
// }
