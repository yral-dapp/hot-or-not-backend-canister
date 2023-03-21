use std::collections::{BTreeMap, HashMap};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use shared_utils::{
    access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMap,
};

use self::canister_upgrade::upgrade_status::UpgradeStatus;

pub mod canister_upgrade;

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct CanisterData {
    pub last_run_upgrade_status: UpgradeStatus,
    pub known_principal_ids: KnownPrincipalMap,
    // TODO: remove this field on next upgrade
    #[serde(skip_serializing)]
    pub access_control_map: HashMap<Principal, Vec<UserAccessRole>>,
    pub user_principal_id_to_canister_id_map: BTreeMap<Principal, Principal>,
    pub unique_user_name_to_user_principal_id_map: BTreeMap<String, Principal>,
}
