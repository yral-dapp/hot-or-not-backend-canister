use candid::Principal;
use shared_utils::common::types::{
    known_principal::KnownPrincipalType, storable_principal::StorablePrincipal,
};

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister(
    unique_user_name_to_user_principal_id_tuple_vec: Vec<(String, Principal)>,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            unique_user_name_to_user_principal_id_tuple_vec,
            &caller_principal_id,
        );
    });
}

fn receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
    canister_data: &mut CanisterData,
    unique_user_name_to_user_principal_id_tuple_vec: Vec<(String, Principal)>,
    caller_principal_id: &Principal,
) {
    let known_principal_ids = &canister_data.heap_data.known_principal_ids;
    let user_index_canister_principal_id = known_principal_ids
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();

    if *user_index_canister_principal_id != *caller_principal_id {
        return;
    }

    let user_principal_id_to_all_user_data_map =
        &mut canister_data.user_principal_id_to_all_user_data_map;

    unique_user_name_to_user_principal_id_tuple_vec
        .iter()
        .for_each(|(unique_user_name, user_principal_id)| {
            match user_principal_id_to_all_user_data_map
                .contains_key(&StorablePrincipal(*user_principal_id))
            {
                true => {
                    let mut existing_entry = user_principal_id_to_all_user_data_map
                        .get(&StorablePrincipal(*user_principal_id))
                        .unwrap();
                    existing_entry.canister_data.unique_user_name = unique_user_name.clone();
                    user_principal_id_to_all_user_data_map
                        .insert(StorablePrincipal(*user_principal_id), existing_entry)
                        .unwrap();
                }
                false => {}
            };
        });
}

#[cfg(test)]
mod test {
    use shared_utils::canister_specific::data_backup::types::all_user_data::{
        AllUserData, UserOwnedCanisterData,
    };
    use test_utils::setup::test_constants::{
        get_mock_canister_id_user_index, get_mock_user_alice_canister_id,
        get_mock_user_alice_principal_id, get_mock_user_bob_canister_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.heap_data.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );
        let alice_unique_user_name_1 = "cool_alice_1234";
        let bob_unique_user_name_1 = "hot_bob_1234";
        let bob_unique_user_name_2 = "handsome_bob_1234";
        let unique_user_name_to_user_principal_id_tuple_vec = vec![
            (
                alice_unique_user_name_1.to_string(),
                get_mock_user_alice_principal_id(),
            ),
            (
                bob_unique_user_name_1.to_string(),
                get_mock_user_bob_principal_id(),
            ),
        ];

        // * Anonymous caller should not be able to call this method.
        let caller_principal_id = Principal::anonymous();
        receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            unique_user_name_to_user_principal_id_tuple_vec.clone(),
            &caller_principal_id,
        );

        assert_eq!(
            canister_data.user_principal_id_to_all_user_data_map.len(),
            0
        );

        // * User Index Canister should be able to call this method.
        let caller_principal_id = get_mock_canister_id_user_index();
        receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            unique_user_name_to_user_principal_id_tuple_vec.clone(),
            &caller_principal_id,
        );

        // * But the user_principal_id -> canister_id mapping should be present
        assert_eq!(
            canister_data.user_principal_id_to_all_user_data_map.len(),
            0
        );

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_alice_canister_id(),
                canister_data: UserOwnedCanisterData {
                    ..Default::default()
                },
            },
        );
        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .contains_key(&StorablePrincipal(get_mock_user_alice_principal_id())));
        let mut entry = canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .unwrap();
        entry.canister_data.unique_user_name = "blah blah".to_string();
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            ""
        );
        canister_data
            .user_principal_id_to_all_user_data_map
            .insert(StorablePrincipal(get_mock_user_alice_principal_id()), entry)
            .unwrap();
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            "blah blah"
        );

        receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            unique_user_name_to_user_principal_id_tuple_vec.clone(),
            &caller_principal_id,
        );
        assert_eq!(
            canister_data.user_principal_id_to_all_user_data_map.len(),
            1
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            alice_unique_user_name_1.to_string()
        );

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_bob_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_bob_principal_id(),
                user_canister_id: get_mock_user_bob_canister_id(),
                canister_data: UserOwnedCanisterData {
                    ..Default::default()
                },
            },
        );

        receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            unique_user_name_to_user_principal_id_tuple_vec,
            &caller_principal_id,
        );
        assert_eq!(
            canister_data.user_principal_id_to_all_user_data_map.len(),
            2
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            alice_unique_user_name_1.to_string()
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_bob_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            bob_unique_user_name_1.to_string()
        );

        // * Overwriting canister IDs should be allowed.
        let caller_principal_id = get_mock_canister_id_user_index();
        let unique_user_name_to_user_principal_id_tuple_vec = vec![(
            bob_unique_user_name_2.to_string(),
            get_mock_user_bob_principal_id(),
        )];
        receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            unique_user_name_to_user_principal_id_tuple_vec,
            &caller_principal_id,
        );

        assert_eq!(
            canister_data.user_principal_id_to_all_user_data_map.len(),
            2
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_bob_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            bob_unique_user_name_2.to_string()
        );
    }
}
