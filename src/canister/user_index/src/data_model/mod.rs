use std::collections::BTreeMap;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use shared_utils::common::types::known_principal::KnownPrincipalMap;

use self::{canister_upgrade::UpgradeStatus, configuration::Configuration};

pub mod canister_upgrade;
pub mod configuration;

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterData {
    pub configuration: Configuration,
    pub last_run_upgrade_status: UpgradeStatus,
    pub known_principal_ids: KnownPrincipalMap,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
}
