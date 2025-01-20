use std::collections::BTreeMap;

use candid::{Nat, Principal};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::individual_user_template::types::pump_n_dump::ParticipatedGameInfo;

#[derive(Serialize, Deserialize)]
pub struct PumpAndDumpGame {
    /// balance that is only available for gameplay
    pub game_only_balance: Nat,
    /// balance that can be swapped for DOLLR
    pub withdrawable_balance: Nat,
    pub referral_reward: Nat,
    pub onboarding_reward: Nat,
    pub games: Vec<ParticipatedGameInfo>,
    pub total_pumps: Nat,
    pub total_dumps: Nat,
    pub net_earnings: Nat,
    // Root canister: dollr locked
    pub liquidity_pools: BTreeMap<Principal, Nat>,
}

impl Default for PumpAndDumpGame {
    fn default() -> Self {
        Self {
            game_only_balance: 0u32.into(),
            withdrawable_balance: 0u32.into(),
            // 1000 gDOLLR
            referral_reward: Nat::from(1e9 as u64),
            // 1000 DOLLR
            onboarding_reward: Nat::from(1e9 as u64),
            liquidity_pools: BTreeMap::new(),
            games: vec![],
            total_pumps: 0u32.into(),
            total_dumps: 0u32.into(),
            net_earnings: 0u32.into(),
        }
    }
}

impl PumpAndDumpGame {
    /// balance the user can use to play the game
    pub fn playable_balance(&self) -> Nat {
        self.game_only_balance.clone() + self.withdrawable_balance.clone()
    }
}