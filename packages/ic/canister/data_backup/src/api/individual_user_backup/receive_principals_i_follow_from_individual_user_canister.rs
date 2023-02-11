use candid::Principal;
use shared_utils::common::types::storable_principal::StorablePrincipal;

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn receive_principals_i_follow_from_individual_user_canister(
    principals_i_follow_from_individual_user_canister: Vec<Principal>,
    canister_owner_principal_id: Principal,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_principals_i_follow_from_individual_user_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            principals_i_follow_from_individual_user_canister,
            &caller_principal_id,
            &canister_owner_principal_id,
        );
    });
}

fn receive_principals_i_follow_from_individual_user_canister_impl(
    canister_data: &mut CanisterData,
    principals_i_follow_from_individual_user_canister: Vec<Principal>,
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

    principals_i_follow_from_individual_user_canister
        .iter()
        .for_each(|followee_principal| {
            // upsert the post details in the user's record.
            existing_entry
                .canister_data
                .principals_i_follow
                .insert(*followee_principal);
        });

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
        get_mock_user_bob_canister_id, get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_receive_principals_i_follow_from_individual_user_canister_impl() {
        let mut canister_data = CanisterData::default();

        let principals_i_follow_from_individual_user_canister = vec![
            get_mock_user_alice_principal_id(),
            get_mock_user_bob_principal_id(),
        ];

        receive_principals_i_follow_from_individual_user_canister_impl(
            &mut canister_data,
            principals_i_follow_from_individual_user_canister.clone(),
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

        receive_principals_i_follow_from_individual_user_canister_impl(
            &mut canister_data,
            principals_i_follow_from_individual_user_canister.clone(),
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
                .principals_i_follow
                .len(),
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

        receive_principals_i_follow_from_individual_user_canister_impl(
            &mut canister_data,
            principals_i_follow_from_individual_user_canister,
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
                .principals_i_follow
                .len(),
            2
        );
    }
}
