use candid::Principal;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_principals_that_follow_me_from_data_backup_canister(
    principals_that_follow_me_chunk_vec: Vec<Principal>,
) {
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
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        for principal_that_follows_me in principals_that_follow_me_chunk_vec {
            canister_data
                .principals_that_follow_me
                .insert(principal_that_follows_me);
        }
    });
}
