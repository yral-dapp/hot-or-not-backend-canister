use shared_utils::canister_specific::user_index::types::args::UserIndexInitArgs;

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::init]
#[candid::candid_method(init)]
fn init(init_args: UserIndexInitArgs) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut data = canister_data_ref_cell.borrow_mut();
        init_impl(init_args, &mut data);
    });
}

fn init_impl(init_args: UserIndexInitArgs, data: &mut CanisterData) {
    init_args
        .known_principal_ids
        .unwrap_or_default()
        .iter()
        .for_each(|(principal_belongs_to, principal_id)| {
            data.known_principal_ids
                .insert(principal_belongs_to.clone(), principal_id.clone());
        });
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use shared_utils::{
        access_control::UserAccessRole,
        common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
    };
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_canister_id_configuration,
        get_mock_canister_id_user_index, get_mock_user_alice_canister_id,
    };

    use super::*;

    #[test]
    fn test_init_impl() {
        // * Add some known principals
        let mut known_principal_ids = KnownPrincipalMap::new();
        known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id_v1(),
        );
        known_principal_ids.insert(
            KnownPrincipalType::CanisterIdConfiguration,
            get_mock_canister_id_configuration(),
        );
        known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );

        // * Add some access control roles
        let mut access_control_map = HashMap::new();
        access_control_map.insert(
            get_global_super_admin_principal_id_v1(),
            vec![
                UserAccessRole::CanisterController,
                UserAccessRole::CanisterAdmin,
            ],
        );
        access_control_map.insert(
            get_mock_user_alice_canister_id(),
            vec![UserAccessRole::ProjectCanister],
        );

        // * Create the init args
        let init_args = UserIndexInitArgs {
            known_principal_ids: Some(known_principal_ids),
            access_control_map: Some(access_control_map),
        };
        let mut data = CanisterData::default();

        // * Run the init impl
        init_impl(init_args, &mut data);

        // * Check the data
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .unwrap(),
            &get_global_super_admin_principal_id_v1()
        );
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::CanisterIdConfiguration)
                .unwrap(),
            &get_mock_canister_id_configuration()
        );
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .unwrap(),
            &get_mock_canister_id_user_index()
        );
    }
}
