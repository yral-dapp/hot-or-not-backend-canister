use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::types::canister_specific::user_index::error_types::SetUniqueUsernameError;

use crate::{data_model::CanisterData, CANISTER_DATA};

#[update]
fn update_index_with_unique_user_name_corresponding_to_user_principal_id(
    unique_user_name: String,
    user_principal_id: Principal,
) -> Result<(), SetUniqueUsernameError> {
    let request_makers_canister_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        update_index_with_unique_user_name_corresponding_to_user_principal_id_impl(
            unique_user_name,
            user_principal_id,
            request_makers_canister_id,
            &mut canister_data_ref_cell.borrow_mut(),
        )
    })
}

fn update_index_with_unique_user_name_corresponding_to_user_principal_id_impl(
    unique_user_name: String,
    user_principal_id: Principal,
    request_makers_canister_id: Principal,
    canister_data: &mut CanisterData,
) -> Result<(), SetUniqueUsernameError> {
    if !canister_data
        .user_principal_id_to_canister_id_map
        .contains_key(&user_principal_id)
    {
        return Err(SetUniqueUsernameError::UserCanisterEntryDoesNotExist);
    }

    if canister_data
        .user_principal_id_to_canister_id_map
        .get(&user_principal_id)
        != Some(&request_makers_canister_id)
    {
        return Err(SetUniqueUsernameError::SendingCanisterDoesNotMatchUserCanisterId);
    }

    if canister_data
        .unique_user_name_to_user_principal_id_map
        .contains_key(&unique_user_name)
    {
        return Err(SetUniqueUsernameError::UsernameAlreadyTaken);
    }

    canister_data
        .unique_user_name_to_user_principal_id_map
        .insert(unique_user_name.clone(), user_principal_id);

    Ok(())
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_canister_id, get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_update_index_with_unique_user_name_corresponding_to_user_principal_id_impl() {
        let unique_user_name_1 = "cool_alice_1234".to_string();
        let unique_user_name_2 = "cool_alice_5678".to_string();
        let user_principal_id = get_mock_user_alice_principal_id();
        let request_makers_canister_id = get_mock_user_alice_canister_id();
        let mut canister_data = CanisterData::default();

        let result = update_index_with_unique_user_name_corresponding_to_user_principal_id_impl(
            unique_user_name_1.clone(),
            user_principal_id,
            request_makers_canister_id,
            &mut canister_data,
        );
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            SetUniqueUsernameError::UserCanisterEntryDoesNotExist
        );

        canister_data
            .user_principal_id_to_canister_id_map
            .insert(user_principal_id, request_makers_canister_id);

        let result = update_index_with_unique_user_name_corresponding_to_user_principal_id_impl(
            unique_user_name_1.clone(),
            user_principal_id,
            get_mock_user_bob_canister_id(),
            &mut canister_data,
        );
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            SetUniqueUsernameError::SendingCanisterDoesNotMatchUserCanisterId
        );

        canister_data
            .unique_user_name_to_user_principal_id_map
            .insert(unique_user_name_1.clone(), get_mock_user_bob_principal_id());

        let result = update_index_with_unique_user_name_corresponding_to_user_principal_id_impl(
            unique_user_name_1.clone(),
            user_principal_id,
            request_makers_canister_id,
            &mut canister_data,
        );
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            SetUniqueUsernameError::UsernameAlreadyTaken
        );

        let result = update_index_with_unique_user_name_corresponding_to_user_principal_id_impl(
            unique_user_name_2.clone(),
            user_principal_id,
            request_makers_canister_id,
            &mut canister_data,
        );
        assert!(result.is_ok());
        assert_eq!(
            canister_data
                .unique_user_name_to_user_principal_id_map
                .get(&unique_user_name_2)
                .unwrap(),
            &user_principal_id
        );
    }
}
