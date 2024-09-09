use std::collections::HashMap;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::canister_specific::user_index::types::UpgradeStatus;

pub mod args;
pub mod well_known_principal;

#[derive(Default, Clone, CandidType, Serialize, Deserialize)]
pub struct SubnetUpgradeReport {
    pub failed_canister_ids: Vec<(Principal, Principal, String)>,
    pub succesful_upgrade_count: u128,
    pub subnet_wise_report: HashMap<Principal, UpgradeStatus>,
}
