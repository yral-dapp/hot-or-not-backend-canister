use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::util::types::individual_user_canister::IndividualUserCanister;

#[update]
async fn allot_number_of_empty_canister(number: u32) -> Result<Principal, String> {
    let registered_individual_canister = IndividualUserCanister::new(caller())?;
    registered_individual_canister
        .allot_number_of_empty_canisters(number)
        .await
}
