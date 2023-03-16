use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::HotOrNotDetails,
    common::{types::top_posts::post_score_index_item::PostScoreIndexItem, utils::system_time},
    constant::{
        HOME_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION,
        HOT_OR_NOT_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION,
    },
};

use crate::CANISTER_DATA;

// TODO: add access control to only allow fleet canisters to call this function
#[ic_cdk::update]
#[candid::candid_method(update)]
pub async fn update_scores_and_share_with_post_cache_if_difference_beyond_threshold(post_id: u64) {
    let current_time = system_time::get_current_system_time_from_ic();

    // let post_to_synchronise: Option<PostScoreIndexItem> =
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_synchronise = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&post_id)
            .unwrap()
            .clone();

        post_to_synchronise.recalculate_home_feed_score(&current_time);

        //         let last_updated_home_feed_score =
        //             post_to_synchronise.last_synchronized_values.home_feed_score;
        //         let current_home_feed_score = post_to_synchronise.homefeed_ranking_score;

        //         post_to_synchronise.recalculate_hot_or_not_feed_score(&current_time);
        //         let home_feed_score_difference =
        //             current_home_feed_score.abs_diff(last_updated_home_feed_score);

        //         let last_updated_hot_or_not_feed_score = post_to_synchronise
        //             .last_synchronized_values
        //             .hot_or_not_feed_score;
        //         let current_hot_or_not_feed_score = post_to_synchronise
        //             .hot_or_not_details
        //             .unwrap_or(HotOrNotDetails::default())
        //             .score;

        //         let hot_or_not_feed_score_difference =
        //             current_hot_or_not_feed_score.abs_diff(last_updated_hot_or_not_feed_score);

        //         if home_feed_score_difference > HOME_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION
        //             || hot_or_not_feed_score_difference
        //                 > HOT_OR_NOT_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION
        //         {
        //             Some(PostScoreIndexItem {
        //                 post_id: post_to_synchronise.id,
        //                 score: current_home_feed_score,
        //                 publisher_canister_id: ic_cdk::id(),
        //             })
        //         } else {
        //             None
        //         }
    });
}
