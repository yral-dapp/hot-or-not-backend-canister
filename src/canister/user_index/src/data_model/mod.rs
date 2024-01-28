use std::collections::{BTreeMap, HashSet};


use candid::{Deserialize, Principal};
use ic_stable_structures::storable::Blob;
use ic_stable_structures::StableBTreeMap;
use serde::Serialize;
use shared_utils::common::types::wasm::WasmType;

use self::memory::get_wasm_memory;
use self::{canister_upgrade::UpgradeStatus, configuration::Configuration, memory::Memory};

pub mod canister_upgrade;
pub mod configuration;
pub mod memory;


const fn _default_true() -> bool {
    return true;
}

fn _default_vec_principal() -> HashSet<Principal> {
    return HashSet::new()
}

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub allow_upgrades_for_individual_canisters: bool,
    pub available_canisters: HashSet<Principal>,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
    #[serde(skip, default = "_empty_wasms")]
    pub wasms: StableBTreeMap<WasmType, Blob<4096>, Memory>
}

impl Default for CanisterData {
    fn default() -> Self {
        Self { configuration: Default::default(), last_run_upgrade_status: Default::default(), allow_upgrades_for_individual_canisters: Default::default(), available_canisters: Default::default(), user_principal_id_to_canister_id_map: Default::default(), unique_user_name_to_user_principal_id_map: Default::default(), wasms: _empty_wasms() }
    }
}

fn _empty_wasms() -> StableBTreeMap<WasmType, Blob<4096>, Memory> {
    StableBTreeMap::init(get_wasm_memory())
}