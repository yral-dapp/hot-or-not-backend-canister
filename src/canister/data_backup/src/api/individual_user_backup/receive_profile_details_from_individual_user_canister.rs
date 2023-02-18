use candid::Principal;
use shared_utils::{
    canister_specific::data_backup::types::all_user_data::{AllUserData, UserOwnedCanisterData},
    common::types::storable_principal::StorablePrincipal,
};

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_profile_details_from_individual_user_canister(
    display_name: Option<String>,
    profile_picture_url: Option<String>,
    unique_user_name: Option<String>,
    canister_owner_principal_id: Principal,
    canister_id: Principal,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_profile_details_from_individual_user_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            &caller_principal_id,
            display_name,
            profile_picture_url,
            unique_user_name,
            &canister_owner_principal_id,
            &canister_id,
        );
    });
}

fn receive_profile_details_from_individual_user_canister_impl(
    canister_data: &mut CanisterData,
    caller_principal_id: &Principal,
    display_name: Option<String>,
    profile_picture_url: Option<String>,
    unique_user_name: Option<String>,
    canister_owner_principal_id: &Principal,
    canister_id: &Principal,
) {
    if *canister_id != *caller_principal_id {
        return;
    }

    let mut entry_to_insert = if canister_data
        .user_principal_id_to_all_user_data_map
        .contains_key(&StorablePrincipal(*canister_owner_principal_id))
    {
        canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(*canister_owner_principal_id))
            .unwrap()
    } else {
        AllUserData {
            user_principal_id: *canister_owner_principal_id,
            user_canister_id: *canister_id,
            canister_data: UserOwnedCanisterData::default(),
        }
    };

    entry_to_insert.canister_data.profile.display_name = display_name;
    entry_to_insert.canister_data.profile.profile_picture_url = profile_picture_url;
    entry_to_insert.canister_data.unique_user_name = unique_user_name.unwrap_or("".to_string());

    canister_data.user_principal_id_to_all_user_data_map.insert(
        StorablePrincipal(*canister_owner_principal_id),
        entry_to_insert,
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
    fn test_receive_profile_details_from_individual_user_canister_impl() {
        let mut canister_data = CanisterData::default();

        let display_name = Some("Alice".to_string());
        let profile_picture_url = Some("https://alice.com".to_string());
        let unique_user_name = Some("alice".to_string());

        receive_profile_details_from_individual_user_canister_impl(
            &mut canister_data,
            &get_mock_user_bob_canister_id(),
            display_name.clone(),
            profile_picture_url.clone(),
            unique_user_name.clone(),
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
        );
        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_none());

        receive_profile_details_from_individual_user_canister_impl(
            &mut canister_data,
            &get_mock_user_alice_canister_id(),
            display_name.clone(),
            profile_picture_url.clone(),
            unique_user_name.clone(),
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
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
                .profile
                .display_name,
            Some("Alice".to_string())
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .profile
                .profile_picture_url,
            Some("https://alice.com".to_string())
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            "alice"
        );

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_bob_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_profile_details_from_individual_user_canister_impl(
            &mut canister_data,
            &get_mock_user_alice_canister_id(),
            display_name.clone(),
            profile_picture_url.clone(),
            unique_user_name.clone(),
            &get_mock_user_alice_principal_id(),
            &get_mock_user_alice_canister_id(),
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
                .profile
                .display_name,
            Some("Alice".to_string())
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .profile
                .profile_picture_url,
            Some("https://alice.com".to_string())
        );
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .unique_user_name,
            "alice"
        );
    }
}
