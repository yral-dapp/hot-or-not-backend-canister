use ic_cdk_macros::update;

use crate::{util::guards::is_caller_profile_owner, CANISTER_DATA};

/// Returns error in case post is not found
#[update(guard = "is_caller_profile_owner")]
fn delete_post(post_id: u64) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.delete_post(post_id))
}
