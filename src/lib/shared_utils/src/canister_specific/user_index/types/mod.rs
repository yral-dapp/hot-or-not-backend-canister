use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

pub mod args;

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq)]
pub struct UpgradeStatus {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    pub failed_canister_ids: Vec<(Principal, Principal, String)>,
    #[serde(default)]
    pub version: String,
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
            version: String::from("v0.0.0"),
        }
    }
}

#[derive(Debug, CandidType, Serialize, Deserialize, Default, Clone)]
pub struct RecycleStatus {
    pub success_canisters: Vec<String>,
    pub num_last_recycled_canisters: u64,
    pub last_recycled_at: Option<SystemTime>,
    pub last_recycled_duration: Option<u64>,
    pub failed_recycling: Vec<(Principal, String)>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct BroadcastCallStatus {
    pub method_name: String,
    pub successful_canister_ids: Vec<Principal>,
    pub failed_canister_ids: Vec<(Principal, String)>,
    pub successful_canisters_count: u64,
    pub failed_canisters_count: u64,
    pub total_canisters: u64,
    pub timestamp: SystemTime,
}

impl Default for BroadcastCallStatus {
    fn default() -> Self {
        Self {
            timestamp: UNIX_EPOCH,
            successful_canister_ids: Vec::new(),
            failed_canister_ids: Vec::new(),
            method_name: String::from(""),
            successful_canisters_count: 0,
            total_canisters: 0,
            failed_canisters_count: 0,
        }
    }
}
