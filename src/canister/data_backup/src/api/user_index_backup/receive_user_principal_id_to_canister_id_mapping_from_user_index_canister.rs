use candid::Principal;
use shared_utils::{
    canister_specific::data_backup::types::all_user_data::{AllUserData, UserOwnedCanisterData},
    common::types::{known_principal::KnownPrincipalType, storable_principal::StorablePrincipal},
};

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_user_principal_id_to_canister_id_mapping_from_user_index_canister(
    user_principal_id_to_canister_id_tuple_vec: Vec<(Principal, Principal)>,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_user_principal_id_to_canister_id_mapping_from_user_index_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            user_principal_id_to_canister_id_tuple_vec,
            &caller_principal_id,
        );
    });
}

fn receive_user_principal_id_to_canister_id_mapping_from_user_index_canister_impl(
    canister_data: &mut CanisterData,
    user_principal_id_to_canister_id_tuple_vec: Vec<(Principal, Principal)>,
    caller_principal_id: &Principal,
) {
    let known_principal_ids = &canister_data.heap_data.known_principal_ids;
    let user_index_canister_principal_id = known_principal_ids
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .expect("Failed to get the canister id of the user_index canister");

    if *user_index_canister_principal_id != *caller_principal_id {
        return;
    }

    let user_principal_id_to_all_user_data_map =
        &mut canister_data.user_principal_id_to_all_user_data_map;

    user_principal_id_to_canister_id_tuple_vec.iter().for_each(
        |(user_principal_id, canister_principal_id)| {
            match user_principal_id_to_all_user_data_map
                .contains_key(&StorablePrincipal(*user_principal_id))
            {
                true => {
                    let mut existing_entry = user_principal_id_to_all_user_data_map
                        .get(&StorablePrincipal(*user_principal_id))
                        .unwrap();
                    existing_entry.user_principal_id = *user_principal_id;
                    existing_entry.user_canister_id = *canister_principal_id;
                    user_principal_id_to_all_user_data_map
                        .insert(StorablePrincipal(*user_principal_id), existing_entry)
                        .unwrap();
                }
                false => {
                    let new_entry = AllUserData {
                        user_principal_id: *user_principal_id,
                        user_canister_id: *canister_principal_id,
                        canister_data: UserOwnedCanisterData::default(),
                    };
                    user_principal_id_to_all_user_data_map
                        .insert(StorablePrincipal(*user_principal_id), new_entry);
                }
            };
        },
    );
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_canister_id_user_index, get_mock_user_alice_canister_id,
        get_mock_user_alice_principal_id, get_mock_user_bob_canister_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_receive_user_principal_id_to_canister_id_mapping_from_user_index_canister_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.heap_data.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );
        let user_principal_id_to_canister_id_tuple_vec = vec![
            (
                get_mock_user_alice_principal_id(),
                get_mock_user_alice_canister_id(),
            ),
            (
                get_mock_user_bob_principal_id(),
                get_mock_user_bob_canister_id(),
            ),
        ];

        // * Anonymous caller should not be able to call this method.
        let caller_principal_id = Principal::anonymous();
        receive_user_principal_id_to_canister_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            user_principal_id_to_canister_id_tuple_vec.clone(),
            &caller_principal_id,
        );

        assert_eq!(
            canister_data.user_principal_id_to_all_user_data_map.len(),
            0
        );

        // * User Index Canister should be able to call this method.
        let caller_principal_id = get_mock_canister_id_user_index();
        receive_user_principal_id_to_canister_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            user_principal_id_to_canister_id_tuple_vec,
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
                .user_canister_id,
            get_mock_user_alice_canister_id()
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_bob_principal_id()))
                .unwrap()
                .user_canister_id,
            get_mock_user_bob_canister_id()
        );

        // * Overwriting canister IDs should be allowed.
        let caller_principal_id = get_mock_canister_id_user_index();
        let user_principal_id_to_canister_id_tuple_vec = vec![(
            get_mock_user_alice_principal_id(),
            get_mock_user_bob_canister_id(),
        )];
        receive_user_principal_id_to_canister_id_mapping_from_user_index_canister_impl(
            &mut canister_data,
            user_principal_id_to_canister_id_tuple_vec,
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
                .user_canister_id,
            get_mock_user_bob_canister_id()
        );
    }
}
