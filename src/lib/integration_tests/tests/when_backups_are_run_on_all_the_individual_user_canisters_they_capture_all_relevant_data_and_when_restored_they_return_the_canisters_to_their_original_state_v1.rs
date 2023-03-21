// use candid::Principal;
// use ic_cdk::api::management_canister::{
//     main::CanisterStatusResponse, provisional::CanisterIdRecord,
// };
// use ic_test_state_machine_client::call_candid_as;
// use shared_utils::common::types::known_principal::KnownPrincipalType;
// use test_utils::setup::{
//     env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
//     test_constants::get_global_super_admin_principal_id_v1,
// };

#[test]
fn when_backups_are_run_on_all_the_individual_user_canisters_they_capture_all_relevant_data_and_when_restored_they_return_the_canisters_to_their_original_state_v1(
) {
    // * Arrange
    // let state_machine = get_new_state_machine();
    // let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    // let user_index_canister_id = known_principal_map
    //     .get(&KnownPrincipalType::CanisterIdUserIndex)
    //     .expect("Canister type not found in principal id map");
    // let data_backup_canister_id = known_principal_map
    //     .get(&KnownPrincipalType::CanisterIdDataBackup)
    //     .expect("Canister type not found in principal id map");

    // TODO: get the rest from the other counterpart test
}
