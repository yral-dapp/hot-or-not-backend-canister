// pub fn update_post_home_feed_score_index_on_home_feed_post_score_recalculation(
//     post_id: u64,
//     new_score: u64,
// ) {
//     let mut posts_index_sorted_by_score = s!(PostsIndexSortedByScore);
//     let mut posts_index_sorted_by_home_feed_score = s!(PostsIndexSortedByHomeFeedScore);

//     let post_score_index_item = PostScoreIndexItem {
//         score: new_score,
//         post_id,
//         publisher_canister_id: SPrincipal(ic_cdk::id()),
//     };
//     posts_index_sorted_by_score.replace(post_score_index_item.clone());
//     posts_index_sorted_by_home_feed_score.replace(&post_score_index_item);

//     // if the index exceeds 150 items, remove the excess
//     if posts_index_sorted_by_score.len() > 150 {
//         posts_index_sorted_by_score = posts_index_sorted_by_score.into_iter().take(100).collect();
//         posts_index_sorted_by_home_feed_score = posts_index_sorted_by_home_feed_score
//             .into_iter()
//             .take(100)
//             .cloned()
//             .collect();
//     }

//     s! { PostsIndexSortedByScore = posts_index_sorted_by_score };
//     s! { PostsIndexSortedByHomeFeedScore = posts_index_sorted_by_home_feed_score };
// }

// pub fn update_post_score_index_on_hot_or_not_feed_post_score_recalculation(
//     post_id: u64,
//     new_score: u64,
// ) {
//     let mut posts_index_sorted_by_hot_or_not_feed_score = s!(PostsIndexSortedByHotOrNotFeedScore);

//     let post_score_index_item = PostScoreIndexItem {
//         score: new_score,
//         post_id,
//         publisher_canister_id: SPrincipal(ic_cdk::id()),
//     };

//     posts_index_sorted_by_hot_or_not_feed_score.replace(&post_score_index_item);

//     if posts_index_sorted_by_hot_or_not_feed_score.iter().count() > 150 {
//         posts_index_sorted_by_hot_or_not_feed_score = posts_index_sorted_by_hot_or_not_feed_score
//             .into_iter()
//             .take(100)
//             .cloned()
//             .collect();
//     }

//     s! { PostsIndexSortedByHotOrNotFeedScore = posts_index_sorted_by_hot_or_not_feed_score };
// }

// pub fn send_top_home_feed_post_scores_to_post_cache_canister() {
//     let top_post_scores: Vec<PostScoreIndexItem> = s!(PostsIndexSortedByHomeFeedScore)
//         .iter()
//         .take(3)
//         .cloned()
//         .collect();

//     let known_principal_ids: MyKnownPrincipalIdsMap = s!(MyKnownPrincipalIdsMap);

//     let _ = call::notify(
//         constant::get_post_cache_canister_principal_id(known_principal_ids),
//         "receive_top_home_feed_posts_from_publishing_canister",
//         (top_post_scores,),
//     );
// }

// pub fn send_top_hot_or_not_feed_post_scores_to_post_cache_canister() {
//     let top_post_scores: Vec<PostScoreIndexItem> = s!(PostsIndexSortedByHotOrNotFeedScore)
//         .iter()
//         .take(3)
//         .cloned()
//         .collect();

//     let known_principal_ids: MyKnownPrincipalIdsMap = s!(MyKnownPrincipalIdsMap);

//     let _ = call::notify(
//         constant::get_post_cache_canister_principal_id(known_principal_ids),
//         "receive_top_hot_or_not_feed_posts_from_publishing_canister",
//         (top_post_scores,),
//     );
// }

// pub fn update_home_feed_post_scores_for_every_post_in_posts_index_sorted_by_home_feed_score(
//     time_provider: &impl Fn() -> SystemTime,
// ) {
//     let posts_index_sorted_by_home_feed_score = s!(PostsIndexSortedByHomeFeedScore);
//     let mut all_created_posts = s!(AllCreatedPostsV1);

//     for post_score_index_item in posts_index_sorted_by_home_feed_score.iter() {
//         let mut post = all_created_posts
//             .get_cloned(post_score_index_item.post_id)
//             .unwrap();

//         post.recalculate_home_feed_score(time_provider);

//         all_created_posts.replace(post_score_index_item.post_id, &post);
//     }

//     s! { AllCreatedPostsV1 = all_created_posts };
// }

// pub fn update_home_feed_post_scores_for_every_post_in_posts_index_sorted_by_home_feed_score_v1() {
//     update_home_feed_post_scores_for_every_post_in_posts_index_sorted_by_home_feed_score_v1_impl(
//         &system_time::get_current_system_time_from_ic,
//     );
// }

// pub fn update_home_feed_post_scores_for_every_post_in_posts_index_sorted_by_home_feed_score_v1_impl(
//     time_provider: &impl Fn() -> SystemTime,
// ) {
//     let posts_index_sorted_by_home_feed_score = s!(PostsIndexSortedByHomeFeedScore);
//     let mut all_created_posts = s!(AllCreatedPostsV1);

//     for post_score_index_item in posts_index_sorted_by_home_feed_score.iter() {
//         let mut post = all_created_posts
//             .get_cloned(post_score_index_item.post_id)
//             .unwrap();

//         post.recalculate_home_feed_score(time_provider);

//         all_created_posts.replace(post_score_index_item.post_id, &post);
//     }

//     s! { AllCreatedPostsV1 = all_created_posts };
// }

// pub fn update_hot_or_not_feed_post_scores_for_every_post_in_posts_index_sorted_by_hot_or_not_feed_score(
//     time_provider: &impl Fn() -> SystemTime,
// ) {
//     let posts_index_sorted_by_hot_or_not_feed_score = s!(PostsIndexSortedByHomeFeedScore);
//     let mut all_created_posts = s!(AllCreatedPostsV1);

//     for post_score_index_item in posts_index_sorted_by_hot_or_not_feed_score.iter() {
//         let mut post = all_created_posts
//             .get_cloned(post_score_index_item.post_id)
//             .unwrap();

//         post.recalculate_hot_or_not_feed_score(time_provider);

//         all_created_posts.replace(post_score_index_item.post_id, &post);
//     }

//     s! { AllCreatedPostsV1 = all_created_posts };
// }

// pub fn update_hot_or_not_feed_post_scores_for_every_post_in_posts_index_sorted_by_hot_or_not_feed_score_v1(
// ) {
//     update_hot_or_not_feed_post_scores_for_every_post_in_posts_index_sorted_by_hot_or_not_feed_score_v1_impl(&system_time::get_current_system_time_from_ic);
// }

// pub fn update_hot_or_not_feed_post_scores_for_every_post_in_posts_index_sorted_by_hot_or_not_feed_score_v1_impl(
//     time_provider: &impl Fn() -> SystemTime,
// ) {
//     let posts_index_sorted_by_hot_or_not_feed_score = s!(PostsIndexSortedByHomeFeedScore);
//     let mut all_created_posts = s!(AllCreatedPostsV1);

//     for post_score_index_item in posts_index_sorted_by_hot_or_not_feed_score.iter() {
//         let mut post = all_created_posts
//             .get_cloned(post_score_index_item.post_id)
//             .unwrap();

//         post.recalculate_hot_or_not_feed_score(time_provider);

//         all_created_posts.replace(post_score_index_item.post_id, &post);
//     }

//     s! { AllCreatedPostsV1 = all_created_posts };
// }
