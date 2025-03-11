use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::post::PostDetailsForFrontend;

use crate::CANISTER_DATA;

#[query]
pub fn get_individual_post_details_by_id(post_id: u64) -> PostDetailsForFrontend {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.get_post_for_frontend(post_id, api_caller))
}
