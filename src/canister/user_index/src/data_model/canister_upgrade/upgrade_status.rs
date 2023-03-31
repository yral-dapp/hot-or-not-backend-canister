use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct UpgradeStatus {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    // TODO: add failure reason
    pub failed_canister_ids: Vec<(Principal, Principal)>,
    // TODO: add a field for canisters that were topped up
}

impl Display for UpgradeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Default for UpgradeStatus {
    fn default() -> Self {
        Self {
            version_number: 0,
            last_run_on: UNIX_EPOCH,
            successful_upgrade_count: 0,
            failed_canister_ids: Vec::new(),
        }
    }
}
