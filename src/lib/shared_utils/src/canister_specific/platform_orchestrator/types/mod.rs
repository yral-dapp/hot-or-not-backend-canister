use std::collections::HashMap;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::canister_specific::user_index::types::UpgradeStatus;

pub mod args;
pub mod well_known_principal;

#[derive(Default, Clone, CandidType, Serialize, Deserialize)]
pub struct SubnetUpgradeReport {
    pub subnet_wise_report: HashMap<Principal, UpgradeStatus>,
}
