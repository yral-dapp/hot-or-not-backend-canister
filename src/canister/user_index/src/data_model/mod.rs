use std::collections::{BTreeMap, HashSet};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use self::{canister_upgrade::UpgradeStatus, configuration::Configuration};

pub mod canister_upgrade;
pub mod configuration;


const fn _default_true() -> bool {
    return true;
}

fn _default_vec_principal() -> HashSet<Principal> {
    return HashSet::new()
}

#[derive(Default, CandidType, Serialize, Deserialize)]
pub struct CanisterData {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub allow_upgrades_for_individual_canisters: bool,
    pub available_canisters: HashSet<Principal>,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
}