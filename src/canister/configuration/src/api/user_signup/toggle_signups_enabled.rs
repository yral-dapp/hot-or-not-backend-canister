use candid::Principal;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::{data::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn toggle_signups_enabled() -> Result<(), String> {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        toggle_signups_enabled_impl(api_caller, &mut canister_data)
    })
}

fn toggle_signups_enabled_impl(
    caller: Principal,
    canister_data: &mut CanisterData,
) -> Result<(), String> {
    let super_admin = canister_data
        .known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .ok_or("Super admin not found in internal records")?;

    if caller != *super_admin {
        return Err("Unauthorized".to_string());
    }

    canister_data.signups_enabled = !canister_data.signups_enabled;

    Ok(())
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id,
    };

    use crate::data::CanisterData;

    use super::*;

    #[test]
    fn test_toggle_signups_enabled_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id_v1(),
        );

        let admin_caller = get_global_super_admin_principal_id_v1();
        // super admin should be allowed to toggle
        let result = toggle_signups_enabled_impl(admin_caller, &mut canister_data);
        assert!(result.is_ok());
        assert_eq!(canister_data.signups_enabled, true);
        let result = toggle_signups_enabled_impl(admin_caller, &mut canister_data);
        assert!(result.is_ok());
        assert_eq!(canister_data.signups_enabled, false);

        // non super admin should not be allowed to toggle
        let non_admin_caller = get_mock_user_alice_principal_id();
        let result = toggle_signups_enabled_impl(non_admin_caller, &mut canister_data);
        assert!(result.is_err());
        assert_eq!(canister_data.signups_enabled, false);
    }
}
