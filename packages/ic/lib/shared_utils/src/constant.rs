use std::time::Duration;

use candid::Principal;

use crate::common::types::known_principal::{
    KnownPrincipalMap, KnownPrincipalMapV1, KnownPrincipalType,
};

pub const DYNAMIC_CANISTER_DEFAULT_CREATION_BALANCE: u64 = 1_000_000_000_000; // 1T Cycles
pub const CYCLES_THRESHOLD_TO_INITIATE_RECHARGE: u128 = 500_000_000_000;
pub const RECHARGE_CYCLES_AMOUNT: u128 = 500_000_000_000;
pub const MINIMUM_CYCLES_TO_REVIVE_CANISTER: u128 = 200_000_000_000; // 0.2T Cycles

pub const MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST: u64 = 10000;
pub const MAX_POSTS_IN_ONE_REQUEST: u64 = 100;
pub const TOP_POSTS_SYNC_INTERVAL: u64 = 1_000_000_000 * 60 * 30; // 30 minutes
pub const TOP_POSTS_SYNC_INTERVAL_DURATION: Duration = Duration::from_secs(30 * 60); // 30 minutes
pub const SCORE_RECALCULATION_SYNC_INTERVAL: u64 = 1_000_000_000 * 60 * 60; // 60 minutes
pub const SCORE_RECALCULATION_SYNC_INTERVAL_DURATION: Duration = Duration::from_secs(60 * 60); // 60 minutes

// * Important Principal IDs

// * Global Owner Principal ID
pub fn get_global_super_admin_principal_id(well_known_canisters: KnownPrincipalMap) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => {
            Principal::from_text("7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae")
                .unwrap()
        }
        _ => {
            well_known_canisters
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .expect("USER ID for global super admin not found")
                .0
        }
    }
}

pub fn get_global_super_admin_principal_id_v1(
    well_known_canisters: KnownPrincipalMapV1,
) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => {
            Principal::from_text("7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae")
                .unwrap()
        }
        _ => well_known_canisters
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .expect("USER ID for global super admin not found")
            .clone(),
    }
}

// * post_cache canister Principal ID
pub fn get_post_cache_canister_principal_id(well_known_canisters: KnownPrincipalMap) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => Principal::from_text("y6yjf-jyaaa-aaaal-qbd6q-cai").unwrap(),
        _ => {
            well_known_canisters
                .get(&KnownPrincipalType::CanisterIdPostCache)
                .expect("Canister Id for post cache canister not found")
                .0
        }
    }
}

// * user_index canister Principal ID
pub fn get_user_index_canister_principal_id(well_known_canisters: KnownPrincipalMap) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap(),
        _ => {
            well_known_canisters
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .expect("Canister Id for user index canister not found")
                .0
        }
    }
}

// * project_member_index canister Principal ID
pub fn get_project_member_index_canister_principal_id(
    well_known_canisters: KnownPrincipalMap,
) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => Principal::from_text("efsfj-sqaaa-aaaap-qatwa-cai").unwrap(),
        _ => {
            well_known_canisters
                .get(&KnownPrincipalType::CanisterIdProjectMemberIndex)
                .expect("Canister Id for project member index canister not found")
                .0
        }
    }
}
