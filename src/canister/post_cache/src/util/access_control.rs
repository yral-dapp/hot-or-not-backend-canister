use std::collections::HashMap;

use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::{
    access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMapV1,
    constant::get_global_super_admin_principal_id_v1,
};

pub fn setup_initial_access_control_v1(
    user_id_access_control_map: &mut HashMap<SPrincipal, Vec<UserAccessRole>>,
    known_principal_ids: &KnownPrincipalMapV1,
) {
    // * add global owner
    user_id_access_control_map.insert(
        SPrincipal(get_global_super_admin_principal_id_v1(
            known_principal_ids.clone(),
        )),
        vec![
            UserAccessRole::CanisterController,
            UserAccessRole::CanisterAdmin,
        ],
    );
}

#[cfg(test)]
mod test {
    use shared_utils::common::types::known_principal::KnownPrincipalType;
    use test_utils::setup::test_constants::get_global_super_admin_principal_id;

    use super::*;

    #[test]
    fn test_setup_initial_access_control_v1() {
        let mut user_id_access_control_map = HashMap::new();
        let mut known_principal_ids = KnownPrincipalMapV1::default();
        let global_super_admin = get_global_super_admin_principal_id();
        known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            global_super_admin.0,
        );

        setup_initial_access_control_v1(&mut user_id_access_control_map, &known_principal_ids);

        assert_eq!(
            user_id_access_control_map.get(&SPrincipal(global_super_admin.0)),
            Some(&vec![
                UserAccessRole::CanisterController,
                UserAccessRole::CanisterAdmin,
            ])
        );
    }
}
