use std::collections::BTreeMap;

use candid::{Nat, Principal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PumpAndDumpGame {
    pub dollr_balance: Nat,
    pub referral_reward: Nat,
    pub onboarding_reward: Nat,
    // Root canister: dollr locked
    pub liquidity_pools: BTreeMap<Principal, Nat>,
}

impl Default for PumpAndDumpGame {
    fn default() -> Self {
        Self {
            dollr_balance: 0u32.into(),
            // 1 DOLLR
            referral_reward: Nat::from(1e8 as u64),
            // 1 DOLLR
            onboarding_reward: Nat::from(1e8 as u64),
            liquidity_pools: BTreeMap::new(),
        }
    }
}