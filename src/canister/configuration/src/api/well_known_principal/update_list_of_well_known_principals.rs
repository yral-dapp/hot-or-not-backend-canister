use candid::Principal;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::{data::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_list_of_well_known_principals(
    principal_type: KnownPrincipalType,
    principal_value: Principal,
) -> Result<(), String> {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        update_list_of_well_known_principals_impl(
            principal_type,
            principal_value,
            &mut canister_data,
            &api_caller,
        )
    })
}

fn update_list_of_well_known_principals_impl(
    principal_type: KnownPrincipalType,
    principal_value: Principal,
    canister_data: &mut CanisterData,
    api_caller: &Principal,
) -> Result<(), String> {
    let super_admin = canister_data
        .known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .ok_or("Super admin not found in internal records")?;

    if api_caller != super_admin {
        return Err("Unauthorized".to_string());
    }

    canister_data
        .known_principal_ids
        .insert(principal_type, principal_value);

    Ok(())
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_canister_id_configuration,
        get_mock_canister_id_post_cache, get_mock_canister_id_user_index,
        get_mock_user_alice_principal_id,
    };

    use super::*;

    #[test]
    fn test_update_list_of_well_known_principals_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdConfiguration,
            get_mock_canister_id_configuration(),
        );
        canister_data.known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id_v1(),
        );

        let admin_api_caller = get_global_super_admin_principal_id_v1();
        let principal_type = KnownPrincipalType::CanisterIdPostCache;
        let principal_value = get_mock_canister_id_post_cache();

        let result = update_list_of_well_known_principals_impl(
            principal_type,
            principal_value,
            &mut canister_data,
            &admin_api_caller,
        );
        assert!(result.is_ok());
        assert_eq!(
            canister_data
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdPostCache),
            Some(&get_mock_canister_id_post_cache())
        );
        assert!(canister_data
            .known_principal_ids
            .contains_key(&KnownPrincipalType::CanisterIdPostCache));

        let non_admin_api_caller = get_mock_user_alice_principal_id();
        let principal_type = KnownPrincipalType::CanisterIdUserIndex;
        let principal_value = get_mock_canister_id_user_index();

        let result = update_list_of_well_known_principals_impl(
            principal_type,
            principal_value,
            &mut canister_data,
            &non_admin_api_caller,
        );

        assert!(result.is_err());
        assert_eq!(
            canister_data
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex),
            None
        );
        assert!(!canister_data
            .known_principal_ids
            .contains_key(&KnownPrincipalType::CanisterIdUserIndex));
    }
}
