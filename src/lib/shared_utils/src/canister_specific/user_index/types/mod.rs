use std::time::{SystemTime, UNIX_EPOCH};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

pub mod args;

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
