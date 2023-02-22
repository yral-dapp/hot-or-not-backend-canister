use shared_utils::{
    canister_specific::individual_user_template::types::profile::UserProfile,
    common::types::known_principal::KnownPrincipalType,
};

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_my_profile_from_data_backup_canister(profile: UserProfile) {
    let caller = ic_cdk::caller();
    let data_backup_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .cloned()
            .unwrap()
    });

    if caller != data_backup_canister_id {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell.borrow_mut().profile = profile;
    });
}
