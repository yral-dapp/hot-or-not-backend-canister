use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

pub mod args;

#[derive(Debug, CandidType, Serialize, Deserialize, Default, Clone)]
pub struct RecycleStatus {
    pub num_last_recycled_canisters: u64,
    pub last_recycled_at: Option<SystemTime>,
    pub failed_recycling: Vec<(Principal, String)>,
}
