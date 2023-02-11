use shared_utils::canister_specific::data_backup::types::backup_statistics::BackupStatistics;

use crate::CANISTER_DATA;

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_current_backup_statistics() -> BackupStatistics {
    BackupStatistics {
        number_of_user_entries: CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow()
                .user_principal_id_to_all_user_data_map
                .len()
        }),
    }
}
