use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{canister_info, CanisterInfoRequest},
    caller, id,
};
use ic_cdk_macros::update;

use crate::{util::types::individual_user_canister::IndividualUserCanister, CANISTER_DATA};

#[update]
pub async fn receive_empty_canister_from_individual_canister(
    canister_ids: Vec<Principal>,
) -> Result<(), String> {
    let _individual_canister = IndividualUserCanister::new(caller())?;

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .backup_canister_pool
            .extend(canister_ids.into_iter());
    });

    Ok(())
}
