use candid::Principal;
use shared_utils::common::types::storable_principal::StorablePrincipal;

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn receive_current_token_balance_from_individual_user_canister(
    utility_token_balance: u64,
    canister_owner_principal_id: Principal,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_current_token_balance_from_individual_user_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            utility_token_balance,
            &caller_principal_id,
            &canister_owner_principal_id,
        );
    });
}

fn receive_current_token_balance_from_individual_user_canister_impl(
    canister_data: &mut CanisterData,
    utility_token_balance: u64,
    caller_principal_id: &Principal,
    canister_owner_principal_id: &Principal,
) {
    let does_the_current_call_makers_record_exist = canister_data
        .user_principal_id_to_all_user_data_map
        .contains_key(&StorablePrincipal(*canister_owner_principal_id));

    if !does_the_current_call_makers_record_exist {
        return;
    }

    let mut existing_entry = canister_data
        .user_principal_id_to_all_user_data_map
        .get(&StorablePrincipal(*canister_owner_principal_id))
        .unwrap();

    if existing_entry.user_canister_id != *caller_principal_id {
        return;
    }

    existing_entry
        .canister_data
        .token_data
        .utility_token_balance = utility_token_balance;

    canister_data.user_principal_id_to_all_user_data_map.insert(
        StorablePrincipal(*canister_owner_principal_id),
        existing_entry,
    );
}

#[cfg(test)]
mod test {
    use shared_utils::canister_specific::data_backup::types::all_user_data::{
        AllUserData, UserOwnedCanisterData,
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_canister_id,
    };

    use super::*;

    #[test]
    fn test_receive_current_token_balance_from_individual_user_canister_impl() {
        let mut canister_data = CanisterData::default();

        receive_current_token_balance_from_individual_user_canister_impl(
            &mut canister_data,
            1500,
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_none());

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_bob_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_current_token_balance_from_individual_user_canister_impl(
            &mut canister_data,
            1500,
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_some());
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .token_data
                .utility_token_balance,
            0
        );

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_alice_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_current_token_balance_from_individual_user_canister_impl(
            &mut canister_data,
            1500,
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_some());
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .token_data
                .utility_token_balance,
            1500
        );
    }
}
