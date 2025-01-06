use std::collections::BTreeMap;

use candid::{Nat, Principal};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::individual_user_template::types::pump_n_dump::ParticipatedGameInfo;

#[derive(Serialize, Deserialize)]
pub struct PumpAndDumpGame {
    pub dollr_balance: Nat,
    pub referral_reward: Nat,
    pub onboarding_reward: Nat,
    pub games: Vec<ParticipatedGameInfo>,
    pub total_pumps: Nat,
    pub total_dumps: Nat,
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
            games: vec![],
            total_pumps: 0u32.into(),
            total_dumps: 0u32.into(),
        }
    }
}