use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(Debug, CandidType, Serialize, Deserialize, Default)]
pub struct RecycleStatus {
    pub num_last_recycled_canisters: u64,
    pub last_recycled_at: Option<SystemTime>,
    pub failed_recycling: Vec<(Principal, String)>,
}
