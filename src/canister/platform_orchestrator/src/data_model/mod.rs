use std::{borrow::Cow, collections::HashSet, time::{SystemTime, UNIX_EPOCH}};
use ic_stable_structures::{StableBTreeMap, StableLog, Storable, storable::Bound};
use ciborium::de;


use candid::{CandidType, Principal};
use serde::{Serialize, Deserialize};
use shared_utils::{canister_specific::platform_orchestrator::types::well_known_principal::PlatformOrchestratorKnownPrincipal, common::types::wasm::{CanisterWasm, WasmType}};

use self::memory::{get_canister_upgrade_log_index_memory, get_canister_upgrade_log_memory, get_subnet_orchestrator_wasm_memory, Memory};

pub mod memory;


#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    pub all_subnet_orchestrator_canisters_list: HashSet<Principal>,
    pub all_post_cache_orchestrator_list: HashSet<Principal>,
    pub subet_orchestrator_with_capacity_left: HashSet<Principal>,
    pub version_detail: VersionDetails,
    #[serde(skip, default = "_default_wasms")]
    pub wasms: StableBTreeMap<WasmType, CanisterWasm, Memory>,
    #[serde(skip, default = "_default_canister_upgrade_log")]
    pub subnet_canister_upgrade_log: StableLog<CanisterUpgradeStatus, Memory, Memory>,
    pub last_subnet_canister_upgrade_status: CanisterUpgradeStatus,
    #[serde(default)]
    pub platform_global_admins: HashSet<Principal>,
    #[serde(default)]
    pub known_principals: PlatformOrchestratorKnownPrincipal,
}

fn _default_wasms() -> StableBTreeMap<WasmType, CanisterWasm, Memory> { 
    StableBTreeMap::init(get_subnet_orchestrator_wasm_memory())
}


fn _default_canister_upgrade_log() -> StableLog<CanisterUpgradeStatus, Memory, Memory> {
    StableLog::init(get_canister_upgrade_log_index_memory(), get_canister_upgrade_log_memory()).unwrap()
}

impl Default for CanisterData {
    fn default() -> Self {
        Self { 
            all_subnet_orchestrator_canisters_list: Default::default(), 
            all_post_cache_orchestrator_list: Default::default(),  
            subet_orchestrator_with_capacity_left: Default::default(), 
            version_detail: Default::default(), wasms: _default_wasms(), 
            subnet_canister_upgrade_log: _default_canister_upgrade_log(), 
            last_subnet_canister_upgrade_status: Default::default(),
            known_principals: Default::default(),
            platform_global_admins: Default::default(),
        }
    }
}


#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct VersionDetails {
    pub version: String,
    pub last_update_on: SystemTime,
}

impl Default for VersionDetails {
    fn default() -> Self {
        Self { version: Default::default(), last_update_on: UNIX_EPOCH }
    }
}



// To store the upgrade arguments and the failed canisters list.

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct CanisterUpgradeStatus {
    pub upgrade_arg: UpgradeCanisterArg,
    pub count: u64,
    pub failures: Vec<(Principal, String)>
}

impl Storable for CanisterUpgradeStatus {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let canister_upgrade_log: CanisterUpgradeStatus = de::from_reader(bytes.as_ref()).unwrap();
        canister_upgrade_log
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Default for CanisterUpgradeStatus {
    fn default() -> Self {
        Self { 
            upgrade_arg: UpgradeCanisterArg {
                canister: WasmType::IndividualUserWasm, 
                version: String::from(""), wasm_blob: vec![]
            }, 
            count: Default::default(), 
            failures: Default::default() 
        }
    }
}


#[derive(CandidType, Deserialize, Serialize , Clone)]
pub struct UpgradeCanisterArg {
    pub canister: WasmType,
    pub version: String,
    pub wasm_blob: Vec<u8>
}