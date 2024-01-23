use std::{collections::HashSet, time::{SystemTime, UNIX_EPOCH}};

use candid::{CandidType, Principal};
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct CanisterData {
    pub all_subnet_orchestrator_canisters_list: HashSet<Principal>,
    pub subet_orchestrator_with_capacity_left: HashSet<Principal>,
    pub version_detail: VersionDetails
}

/**
 * all_subnet contains all the canisters
 * subnet_orchestrator_with_capacity_left are the subnet orchestrator canisters
 */

 /**
  * Consistent Hashing 
    Deterministically find the subnet principal from user-principal. So that given a user principal id we can tell the canister-id
  */

impl Default for CanisterData {
    fn default() -> Self {
        Self { all_subnet_orchestrator_canisters_list: Default::default(), subet_orchestrator_with_capacity_left: Default::default(), version_detail: Default::default() }
    }
}


#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct VersionDetails {
    pub version: String,
    pub last_update_on: SystemTime
}

impl Default for VersionDetails {
    fn default() -> Self {
        Self { version: Default::default(), last_update_on: UNIX_EPOCH }
    }
}

