use std::collections::{BTreeMap, HashSet};

use candid::{Deserialize, Principal};
use ic_stable_structures::StableBTreeMap;
use serde::Serialize;
use shared_utils::canister_specific::user_index::types::{
    BroadcastCallStatus, RecycleStatus, UpgradeStatus,
};
use shared_utils::common::participant_crypto::merkle::ChildrenMerkle;
use shared_utils::common::participant_crypto::{ProofOfParticipation, ProofOfParticipationDeriverStore};
use shared_utils::common::types::wasm::{CanisterWasm, WasmType};

use self::memory::get_wasm_memory;
use self::{configuration::Configuration, memory::Memory};

pub mod configuration;
pub mod memory;

const fn _default_true() -> bool {
    return true;
}

fn _default_vec_principal() -> HashSet<Principal> {
    return HashSet::new();
}

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub allow_upgrades_for_individual_canisters: bool,
    pub available_canisters: HashSet<Principal>,
    #[serde(default)]
    backup_canister_pool: HashSet<Principal>,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
    #[serde(skip, default = "_empty_wasms")]
    pub wasms: StableBTreeMap<WasmType, CanisterWasm, Memory>,
    #[serde(default)]
    pub recycle_status: RecycleStatus,
    #[serde(default)]
    pub last_broadcast_call_status: BroadcastCallStatus,
    #[serde(default)]
    pub proof_of_participation: Option<ProofOfParticipation>,
    #[serde(default)]
    pub children_merkle: ChildrenMerkle,
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            configuration: Default::default(),
            last_run_upgrade_status: Default::default(),
            allow_upgrades_for_individual_canisters: Default::default(),
            available_canisters: Default::default(),
            user_principal_id_to_canister_id_map: Default::default(),
            unique_user_name_to_user_principal_id_map: Default::default(),
            wasms: _empty_wasms(),
            backup_canister_pool: Default::default(),
            recycle_status: Default::default(),
            last_broadcast_call_status: Default::default(),
            proof_of_participation: None,
            children_merkle: ChildrenMerkle::default(),
        }
    }
}

impl CanisterData {
    pub fn insert_backup_canister(&mut self, canister_id: Principal) -> bool {
        let inserted = self.backup_canister_pool.insert(canister_id);
        if inserted {
            self.children_merkle.insert_children([canister_id]);
        }
        inserted
    }

    pub fn remove_backup_canister(&mut self, canister_id: &Principal) {
        self.backup_canister_pool.remove(&canister_id);
        // removal from backup pool does not mean its not part of our fleet
        // an individual canister might be installed on it instead, for example
        // so we don't remove it from children_merkle
    }

    // reinsert a previously removed backup canister
    // caller MUST be careful and only call this function if the canister was previously removed
    // USE [`CanisterData::insert_backup_canister`] if you are not sure
    pub fn reinsert_backup_canister_due_to_failure(&mut self, canister_id: Principal) {
        self.backup_canister_pool.insert(canister_id);
    }

    pub fn backup_canisters(&self) -> &HashSet<Principal> {
        &self.backup_canister_pool
    }
}

impl ProofOfParticipationDeriverStore for CanisterData {
    fn children_merkle(&self) -> &ChildrenMerkle {
        &self.children_merkle
    }

    fn children_merkle_mut(&mut self) -> &mut ChildrenMerkle {
        &mut self.children_merkle 
    }
}

fn _empty_wasms() -> StableBTreeMap<WasmType, CanisterWasm, Memory> {
    StableBTreeMap::init(get_wasm_memory())
}
