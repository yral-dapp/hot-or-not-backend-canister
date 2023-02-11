use std::time::{SystemTime, UNIX_EPOCH};

use candid::{CandidType, Deserialize, Principal};
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::date_time::system_time;
use speedy::{Readable, Writable};

#[derive(CandidType, Readable, Writable, Deserialize)]
pub struct UpgradeStatus {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    pub failed_canister_ids: Vec<(SPrincipal, SPrincipal)>,
}

impl Default for UpgradeStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl UpgradeStatus {
    pub fn new() -> Self {
        Self {
            version_number: 0,
            last_run_on: system_time::get_current_system_time_from_ic(),
            successful_upgrade_count: 0,
            failed_canister_ids: Vec::new(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct UpgradeStatusV1 {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    pub failed_canister_ids: Vec<(Principal, Principal)>,
}

impl Default for UpgradeStatusV1 {
    fn default() -> Self {
        Self {
            version_number: 0,
            last_run_on: UNIX_EPOCH,
            successful_upgrade_count: 0,
            failed_canister_ids: Vec::new(),
        }
    }
}

impl From<UpgradeStatus> for UpgradeStatusV1 {
    fn from(item: UpgradeStatus) -> Self {
        Self {
            version_number: item.version_number,
            last_run_on: item.last_run_on,
            successful_upgrade_count: item.successful_upgrade_count,
            failed_canister_ids: item
                .failed_canister_ids
                .iter()
                .map(|(principal, principal_1)| (principal.0, principal_1.0))
                .collect(),
        }
    }
}
