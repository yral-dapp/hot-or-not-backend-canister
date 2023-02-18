use candid::Principal;

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_user_canister_id_from_unique_user_name(user_name: String) -> Option<Principal> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_user_canister_id_from_unique_user_name_impl(user_name, &canister_data_ref_cell.borrow())
    })
}

fn get_user_canister_id_from_unique_user_name_impl(
    user_name: String,
    canister_data: &CanisterData,
) -> Option<Principal> {
    let profile_principal_id = canister_data
        .unique_user_name_to_user_principal_id_map
        .get(&user_name)
        .cloned()?;

    canister_data
        .user_principal_id_to_canister_id_map
        .get(&profile_principal_id)
        .cloned()
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
    };

    use super::*;

    #[test]
    fn test_get_user_canister_id_from_unique_user_name_impl() {
        let mut canister_data = CanisterData::default();
        let alice_user_name = "cool_alice_1234".to_string();

        let result = get_user_canister_id_from_unique_user_name_impl(
            alice_user_name.clone(),
            &canister_data,
        );
        assert_eq!(result, None);

        canister_data
            .unique_user_name_to_user_principal_id_map
            .insert(alice_user_name.clone(), get_mock_user_alice_principal_id());
        let result = get_user_canister_id_from_unique_user_name_impl(
            alice_user_name.clone(),
            &canister_data,
        );
        assert_eq!(result, None);

        canister_data.user_principal_id_to_canister_id_map.insert(
            get_mock_user_alice_principal_id(),
            get_mock_user_alice_canister_id(),
        );
        let result = get_user_canister_id_from_unique_user_name_impl(
            alice_user_name.clone(),
            &canister_data,
        );
        assert_eq!(result, Some(get_mock_user_alice_canister_id()));
    }
}
