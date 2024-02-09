use candid::Principal;

use crate::common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType};

pub const INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT: u128 = 200_000_000_000; // 0.2T Cycles
pub const CYCLES_THRESHOLD_TO_INITIATE_RECHARGE: u128 = 100_000_000_000; // 0.1T Cycles

pub const SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES: u128 =  2_500_000_000_000_000; //2.5kT Cycles
pub const SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD: u128 = 1_000_000_000_000_000; //1kT Cycles
pub const POST_CACHE_CANISTER_CYCLES_RECHARGE_AMOUMT: u128 = 5_000_000_000_000; //5T Cycles
pub const POST_CACHE_CANISTER_CYCLES_THRESHOLD: u128 = 2_000_000_000_000; //2T Cycles

pub const MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST: u64 = 10000;
pub const MAX_POSTS_IN_ONE_REQUEST: u64 = 100;
pub const HOME_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION: u64 = 100;
pub const HOT_OR_NOT_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION: u64 = 100;

pub const INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE: u64 = 10_000;
pub const INDIVIDUAL_USER_CANISTER_SUBNET_THREESHOLD: u64 = 3_000;
pub const INDIVIDUAL_USER_CANISTER_SUBNET_MAX_CAPACITY: u64 = 50_000;
// * Important Principal IDs
pub const GOVERNANCE_CANISTER_ID: &str = "6wcax-haaaa-aaaaq-aaava-cai";
pub const NNS_CYCLE_MINTING_CANISTER: &str = "rkp4c-7iaaa-aaaaa-aaaca-cai";
pub const NNS_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub fn get_global_super_admin_principal_id_v1(
    well_known_canisters: KnownPrincipalMap,
) -> Principal {
    match option_env!("DFX_NETWORK") {
        Some("ic") => {
            Principal::from_text("7gaq2-4kttl-vtbt4-oo47w-igteo-cpk2k-57h3p-yioqe-wkawi-wz45g-jae")
                .unwrap()
        }
        _ => *well_known_canisters
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .expect("USER ID for global super admin not found"),
    }
}
