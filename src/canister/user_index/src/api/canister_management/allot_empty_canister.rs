use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::util::types::registered_individual_user_canister::RegisteredIndividualUserCanister;

#[update]
async fn allot_empty_canister() -> Result<Principal, String> {
    let registered_individual_canister = RegisteredIndividualUserCanister::new(caller())?;
    registered_individual_canister.allot_empty_canister().await
}
