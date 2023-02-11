use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use ic_stable_memory::{collections::hash_map::SHashMap, utils::ic_types::SPrincipal};
use serde::Serialize;
use speedy::{Readable, Writable};

/// The different user roles to be used in access control for principals
/// making calls to a canister
#[derive(Readable, Writable, PartialEq, Eq, Debug, CandidType, Deserialize, Clone, Serialize)]
pub enum UserAccessRole {
    /// User has canister WASM install/uninstall/delete capabilities
    CanisterController,
    /// User has edit access to all data residing in the canister
    CanisterAdmin,
    /// Data in this canister is the data of this user
    ProfileOwner,
    /// This principal is for a canister part of this project
    ProjectCanister,
}

/// Helper method to check if a principal has requisite access
/// to make the canister call
pub fn does_principal_have_role(
    user_id_access_control_map: &SHashMap<SPrincipal, Vec<UserAccessRole>>,
    role_required: UserAccessRole,
    principal: SPrincipal,
) -> bool {
    match user_id_access_control_map.get_cloned(&principal) {
        Some(roles) => roles.contains(&role_required),
        None => false,
    }
}

pub fn does_principal_have_role_v1(
    user_id_access_control_map: &HashMap<SPrincipal, Vec<UserAccessRole>>,
    role_required: UserAccessRole,
    principal: SPrincipal,
) -> bool {
    match user_id_access_control_map.get(&principal) {
        Some(roles) => roles.contains(&role_required),
        None => false,
    }
}

pub fn does_principal_have_role_v2(
    user_id_access_control_map: &HashMap<Principal, Vec<UserAccessRole>>,
    role_required: UserAccessRole,
    principal: Principal,
) -> bool {
    match user_id_access_control_map.get(&principal) {
        Some(roles) => roles.contains(&role_required),
        None => false,
    }
}

pub fn add_role_to_principal_id(
    user_id_access_control_map: &mut SHashMap<SPrincipal, Vec<UserAccessRole>>,
    user_id: SPrincipal,
    role: UserAccessRole,
) {
    assert!(does_principal_have_role(
        user_id_access_control_map,
        UserAccessRole::CanisterAdmin,
        SPrincipal(ic_cdk::caller())
    ));

    let mut roles = user_id_access_control_map
        .get_cloned(&user_id)
        .unwrap_or(vec![]);
    roles.push(role);

    user_id_access_control_map.insert(user_id, &roles);
}

pub fn add_role_to_principal_id_v1(
    user_id_access_control_map: &mut HashMap<SPrincipal, Vec<UserAccessRole>>,
    user_id: SPrincipal,
    role: UserAccessRole,
    caller: Principal,
) {
    if !does_principal_have_role_v1(
        user_id_access_control_map,
        UserAccessRole::CanisterAdmin,
        SPrincipal(caller),
    ) {
        return;
    }

    user_id_access_control_map
        .entry(user_id)
        .and_modify(|r| {
            r.push(role.clone());
        })
        .or_insert(vec![role]);
}

pub fn add_role_to_principal_id_v2(
    user_id_access_control_map: &mut HashMap<Principal, Vec<UserAccessRole>>,
    user_id: Principal,
    role: UserAccessRole,
    caller: Principal,
) {
    if !does_principal_have_role_v2(
        user_id_access_control_map,
        UserAccessRole::CanisterAdmin,
        caller,
    ) {
        return;
    }

    user_id_access_control_map
        .entry(user_id)
        .and_modify(|r| {
            r.push(role.clone());
        })
        .or_insert(vec![role]);
}

pub fn remove_role_from_principal_id(
    user_id_access_control_map: &mut SHashMap<SPrincipal, Vec<UserAccessRole>>,
    user_id: SPrincipal,
    role: UserAccessRole,
) {
    assert!(does_principal_have_role(
        user_id_access_control_map,
        UserAccessRole::CanisterAdmin,
        SPrincipal(ic_cdk::caller())
    ));

    let mut roles = user_id_access_control_map
        .get_cloned(&user_id)
        .unwrap_or(vec![]);
    roles.retain(|r| r != &role);

    user_id_access_control_map.insert(user_id, &roles);
}

pub fn remove_role_from_principal_id_v1(
    user_id_access_control_map: &mut HashMap<SPrincipal, Vec<UserAccessRole>>,
    user_id: SPrincipal,
    role: UserAccessRole,
    caller: Principal,
) {
    if !does_principal_have_role_v1(
        user_id_access_control_map,
        UserAccessRole::CanisterAdmin,
        SPrincipal(caller),
    ) {
        return;
    }

    user_id_access_control_map.entry(user_id).and_modify(|r| {
        r.retain(|x| x != &role);
    });
}

pub fn remove_role_from_principal_id_v2(
    user_id_access_control_map: &mut HashMap<Principal, Vec<UserAccessRole>>,
    user_id: Principal,
    role: UserAccessRole,
    caller: Principal,
) {
    if !does_principal_have_role_v2(
        user_id_access_control_map,
        UserAccessRole::CanisterAdmin,
        caller,
    ) {
        return;
    }

    user_id_access_control_map.entry(user_id).and_modify(|r| {
        r.retain(|x| x != &role);
    });
}

pub fn get_role_for_principal_id(
    user_id_access_control_map: &SHashMap<SPrincipal, Vec<UserAccessRole>>,
    user_id: SPrincipal,
) -> Vec<UserAccessRole> {
    user_id_access_control_map
        .get_cloned(&user_id)
        .unwrap_or(vec![])
}

pub fn get_roles_for_principal_id_v1(
    user_id_access_control_map: &HashMap<SPrincipal, Vec<UserAccessRole>>,
    user_id: SPrincipal,
) -> Vec<UserAccessRole> {
    (user_id_access_control_map.get(&user_id).unwrap_or(&vec![])).to_vec()
}

pub fn get_roles_for_principal_id_v2(
    user_id_access_control_map: &HashMap<Principal, Vec<UserAccessRole>>,
    user_id: Principal,
) -> Vec<UserAccessRole> {
    (user_id_access_control_map.get(&user_id).unwrap_or(&vec![])).to_vec()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use test_utils::setup::test_constants::{
        get_alice_principal_id, get_bob_principal_id, get_global_super_admin_principal_id,
        get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_does_principal_have_role_v1() {
        let mut user_id_access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>> =
            HashMap::new();
        let user_id = SPrincipal(get_global_super_admin_principal_id().0);
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(user_id, roles);

        let result = does_principal_have_role_v1(
            &user_id_access_control_map,
            UserAccessRole::CanisterAdmin,
            user_id,
        );
        assert_eq!(result, true);
    }

    #[test]
    fn test_does_principal_have_role_v2() {
        let mut user_id_access_control_map: HashMap<Principal, Vec<UserAccessRole>> =
            HashMap::new();
        let user_id = get_global_super_admin_principal_id_v1();
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(user_id, roles);

        let result = does_principal_have_role_v2(
            &user_id_access_control_map,
            UserAccessRole::CanisterAdmin,
            user_id,
        );
        assert_eq!(result, true);
    }

    #[test]
    fn test_add_role_to_principal_id_v1() {
        let mut user_id_access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>> =
            HashMap::new();
        let super_admin = SPrincipal(get_global_super_admin_principal_id().0);
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(super_admin, roles);

        // * adds role when called from a canister admin
        let user_to_add_to = SPrincipal(get_alice_principal_id().0);
        let role_to_add = UserAccessRole::ProfileOwner;
        add_role_to_principal_id_v1(
            &mut user_id_access_control_map,
            user_to_add_to,
            role_to_add,
            get_global_super_admin_principal_id().0,
        );

        let result = get_roles_for_principal_id_v1(&user_id_access_control_map, user_to_add_to);
        assert!(result.contains(&UserAccessRole::ProfileOwner));

        // * does not add role when called from a non-canister admin
        let user_to_add = SPrincipal(get_alice_principal_id().0);
        let role_to_add = UserAccessRole::CanisterAdmin;
        add_role_to_principal_id_v1(
            &mut user_id_access_control_map,
            user_to_add,
            role_to_add,
            get_bob_principal_id().0,
        );
        let result = get_roles_for_principal_id_v1(&user_id_access_control_map, user_to_add);
        assert!(!result.contains(&UserAccessRole::CanisterAdmin));
    }

    #[test]
    fn test_add_role_to_principal_id_v2() {
        let mut user_id_access_control_map: HashMap<Principal, Vec<UserAccessRole>> =
            HashMap::new();
        let super_admin = get_global_super_admin_principal_id_v1();
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(super_admin, roles);

        // * adds role when called from a canister admin
        let user_to_add_to = get_mock_user_alice_principal_id();
        let role_to_add = UserAccessRole::ProfileOwner;
        add_role_to_principal_id_v2(
            &mut user_id_access_control_map,
            user_to_add_to,
            role_to_add,
            get_global_super_admin_principal_id().0,
        );

        let result = get_roles_for_principal_id_v2(&user_id_access_control_map, user_to_add_to);
        assert!(result.contains(&UserAccessRole::ProfileOwner));

        // * does not add role when called from a non-canister admin
        let user_to_add = get_mock_user_alice_principal_id();
        let role_to_add = UserAccessRole::CanisterAdmin;
        add_role_to_principal_id_v2(
            &mut user_id_access_control_map,
            user_to_add,
            role_to_add,
            get_bob_principal_id().0,
        );
        let result = get_roles_for_principal_id_v2(&user_id_access_control_map, user_to_add);
        assert!(!result.contains(&UserAccessRole::CanisterAdmin));
    }

    #[test]
    fn test_remove_role_from_principal_id_v1() {
        let mut user_id_access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>> =
            HashMap::new();
        let super_admin = SPrincipal(get_global_super_admin_principal_id().0);
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(super_admin, roles);

        // * removes role when called from a canister admin
        let user_to_remove_from = SPrincipal(get_alice_principal_id().0);
        let role_to_remove = UserAccessRole::ProfileOwner;
        let roles = vec![UserAccessRole::ProfileOwner, UserAccessRole::CanisterAdmin];
        user_id_access_control_map.insert(user_to_remove_from, roles);

        remove_role_from_principal_id_v1(
            &mut user_id_access_control_map,
            user_to_remove_from,
            role_to_remove,
            get_global_super_admin_principal_id().0,
        );

        let result =
            get_roles_for_principal_id_v1(&user_id_access_control_map, user_to_remove_from);
        assert!(!result.contains(&UserAccessRole::ProfileOwner));

        // * does not remove role when called from a non-canister admin
        let user_to_remove_from = SPrincipal(get_alice_principal_id().0);
        let role_to_remove = UserAccessRole::ProfileOwner;
        let roles = vec![UserAccessRole::ProfileOwner, UserAccessRole::CanisterAdmin];
        user_id_access_control_map.insert(user_to_remove_from, roles);

        remove_role_from_principal_id_v1(
            &mut user_id_access_control_map,
            user_to_remove_from,
            role_to_remove,
            get_bob_principal_id().0,
        );

        let result =
            get_roles_for_principal_id_v1(&user_id_access_control_map, user_to_remove_from);
        assert!(result.contains(&UserAccessRole::ProfileOwner));
    }

    #[test]
    fn test_remove_role_from_principal_id_v2() {
        let mut user_id_access_control_map: HashMap<Principal, Vec<UserAccessRole>> =
            HashMap::new();
        let super_admin = get_global_super_admin_principal_id_v1();
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(super_admin, roles);

        // * removes role when called from a canister admin
        let user_to_remove_from = get_mock_user_alice_principal_id();
        let role_to_remove = UserAccessRole::ProfileOwner;
        let roles = vec![UserAccessRole::ProfileOwner, UserAccessRole::CanisterAdmin];
        user_id_access_control_map.insert(user_to_remove_from, roles);

        remove_role_from_principal_id_v2(
            &mut user_id_access_control_map,
            user_to_remove_from,
            role_to_remove,
            get_global_super_admin_principal_id_v1(),
        );

        let result =
            get_roles_for_principal_id_v2(&user_id_access_control_map, user_to_remove_from);
        assert!(!result.contains(&UserAccessRole::ProfileOwner));

        // * does not remove role when called from a non-canister admin
        let user_to_remove_from = get_mock_user_alice_principal_id();
        let role_to_remove = UserAccessRole::ProfileOwner;
        let roles = vec![UserAccessRole::ProfileOwner, UserAccessRole::CanisterAdmin];
        user_id_access_control_map.insert(user_to_remove_from, roles);

        remove_role_from_principal_id_v2(
            &mut user_id_access_control_map,
            user_to_remove_from,
            role_to_remove,
            get_mock_user_bob_principal_id(),
        );

        let result =
            get_roles_for_principal_id_v2(&user_id_access_control_map, user_to_remove_from);
        assert!(result.contains(&UserAccessRole::ProfileOwner));
    }

    #[test]
    fn test_get_role_for_principal_id_v1() {
        let mut user_id_access_control_map: HashMap<SPrincipal, Vec<UserAccessRole>> =
            HashMap::new();
        let user_id = SPrincipal(get_global_super_admin_principal_id().0);
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(user_id, roles);

        let result = get_roles_for_principal_id_v1(&user_id_access_control_map, user_id);
        assert_eq!(result, vec![UserAccessRole::CanisterAdmin]);
    }

    #[test]
    fn test_get_role_for_principal_id_v2() {
        let mut user_id_access_control_map: HashMap<Principal, Vec<UserAccessRole>> =
            HashMap::new();
        let user_id = get_global_super_admin_principal_id_v1();
        let role = UserAccessRole::CanisterAdmin;
        let roles = vec![role];
        user_id_access_control_map.insert(user_id, roles);

        let result = get_roles_for_principal_id_v2(&user_id_access_control_map, user_id);
        assert_eq!(result, vec![UserAccessRole::CanisterAdmin]);
    }
}
