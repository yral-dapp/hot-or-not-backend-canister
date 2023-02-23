use std::time::{SystemTime, UNIX_EPOCH};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Clone, Serialize)]
pub struct UpgradeStatus {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    pub failed_canister_ids: Vec<(Principal, Principal)>,
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
