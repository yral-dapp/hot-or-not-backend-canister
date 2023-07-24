use candid::Principal;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_data_from_backup_canister_and_restore_data_to_heap(
    user_principal_id: Principal,
    user_canister_id: Principal,
    unique_user_name: String,
) {
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_data_from_backup_canister_and_restore_data_to_heap_impl(
            caller_principal_id,
            &mut canister_data_ref_cell.borrow_mut(),
            user_principal_id,
            user_canister_id,
            unique_user_name,
        );
    });
}

fn receive_data_from_backup_canister_and_restore_data_to_heap_impl(
    caller_principal_id: Principal,
    canister_data: &mut CanisterData,
    user_principal_id: Principal,
    user_canister_id: Principal,
    unique_user_name: String,
) {
    if *canister_data
        .known_principal_ids
        .get(&KnownPrincipalType::CanisterIdDataBackup)
        .unwrap()
        != caller_principal_id
    {
        return;
    }

    canister_data
        .user_principal_id_to_canister_id_map
        .insert(user_principal_id, user_canister_id);

    if !unique_user_name.trim().is_empty() {
        canister_data
            .unique_user_name_to_user_principal_id_map
            .insert(unique_user_name, user_principal_id);
    }
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_canister_id_data_backup, get_mock_user_alice_canister_id,
        get_mock_user_alice_principal_id, get_mock_user_bob_canister_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_receive_data_from_backup_canister_and_restore_data_to_heap_impl() {
        let mut canister_data = CanisterData::default();

        canister_data.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdDataBackup,
            get_mock_canister_id_data_backup(),
        );

        receive_data_from_backup_canister_and_restore_data_to_heap_impl(
            Principal::anonymous(),
            &mut canister_data,
            get_mock_user_alice_principal_id(),
            get_mock_user_alice_canister_id(),
            "cool_alice_1234".to_string(),
        );
        receive_data_from_backup_canister_and_restore_data_to_heap_impl(
            Principal::anonymous(),
            &mut canister_data,
            get_mock_user_bob_principal_id(),
            get_mock_user_bob_canister_id(),
            "hot_bob_5678".to_string(),
        );

        assert_eq!(canister_data.user_principal_id_to_canister_id_map.len(), 0);
        assert_eq!(
            canister_data
                .unique_user_name_to_user_principal_id_map
                .len(),
            0
        );

        receive_data_from_backup_canister_and_restore_data_to_heap_impl(
            get_mock_canister_id_data_backup(),
            &mut canister_data,
            get_mock_user_alice_principal_id(),
            get_mock_user_alice_canister_id(),
            "cool_alice_1234".to_string(),
        );
        receive_data_from_backup_canister_and_restore_data_to_heap_impl(
            get_mock_canister_id_data_backup(),
            &mut canister_data,
            get_mock_user_bob_principal_id(),
            get_mock_user_bob_canister_id(),
            "hot_bob_5678".to_string(),
        );

        assert_eq!(canister_data.user_principal_id_to_canister_id_map.len(), 2);
        assert_eq!(
            canister_data
                .unique_user_name_to_user_principal_id_map
                .len(),
            2
        );
    }
}
