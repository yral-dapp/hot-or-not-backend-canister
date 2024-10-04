use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::util::canister_management::provision_number_of_empty_canisters;

#[update(guard = "is_caller_controller")]
async fn provision_empty_canisters(number_of_canisters: u64) {
    ic_cdk::spawn(provision_number_of_empty_canisters(
        number_of_canisters,
        || false,
    ));
}
