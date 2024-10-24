use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{update_settings, CanisterSettings, UpdateSettingsArgument},
    caller,
};
use ic_cdk_macros::update;
use shared_utils::constant::{
    get_backup_individual_user_canister_batch_size, get_backup_individual_user_canister_threshold,
};

use crate::{
    util::{
        canister_management::provision_number_of_empty_canisters,
        types::individual_user_canister::IndividualUserCanister,
    },
    CANISTER_DATA,
};

#[update]
async fn allot_empty_canister() -> Result<Principal, String> {
    let registered_individual_canister = IndividualUserCanister::new(caller())?;
    let result = registered_individual_canister.allot_empty_canister().await;

    let backup_canister_count =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.backup_canister_pool.len() as u64);

    if backup_canister_count < get_backup_individual_user_canister_threshold() {
        let number_of_canisters = get_backup_individual_user_canister_batch_size();
        let breaking_condition = || {
            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data.backup_canister_pool.len() as u64
                    > get_backup_individual_user_canister_batch_size()
            })
        };
        ic_cdk::spawn(provision_number_of_empty_canisters(
            number_of_canisters,
            breaking_condition,
        ));
    }

    result
}
