use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    types::utility_token::token_event::TokenEvent,
};

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_my_utility_token_transaction_history_from_data_backup_canister(
    all_token_events_chunk_vec: Vec<(u64, TokenEvent)>,
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

        for (id, token_event) in all_token_events_chunk_vec {
            canister_data
                .my_token_balance
                .utility_token_transaction_history
                .insert(id, token_event);
        }
    });
}
