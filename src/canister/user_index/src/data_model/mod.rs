use std::collections::{BTreeMap, HashSet};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use shared_utils::common::types::known_principal::{KnownPrincipalMap, self};

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
#[serde(from = "CanisterDataV1")]
pub struct CanisterData {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub allow_upgrades_for_individual_canisters: bool,
    pub available_canisters: HashSet<Principal>,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
}

impl From<CanisterDataV1> for CanisterData {
    fn from(value: CanisterDataV1) -> Self {
        let mut canister_data = CanisterData {
            configuration: value.configuration,
            last_run_upgrade_status: value.last_run_upgrade_status,
            allow_upgrades_for_individual_canisters: value.allow_upgrades_for_individual_canisters,
            user_principal_id_to_canister_id_map: value.user_principal_id_to_canister_id_map,
            unique_user_name_to_user_principal_id_map: value.unique_user_name_to_user_principal_id_map,
            available_canisters: value.available_canisters

        };
        canister_data.configuration.known_principal_ids = value.known_principal_ids;
        
        canister_data
    }
}



#[derive(Deserialize)]
pub struct CanisterDataV1 {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub allow_upgrades_for_individual_canisters: bool,
    pub available_canisters: HashSet<Principal>,
    pub known_principal_ids: KnownPrincipalMap,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
}