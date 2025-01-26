use std::borrow::Cow;

use candid::{Nat, Principal};
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::individual_user_template::types::pump_n_dump::ParticipatedGameInfo;

use super::memory::{get_lp_memory, Memory};

pub fn _default_lp(
) -> StableBTreeMap<Principal, NatStore, Memory> {
    StableBTreeMap::init(get_lp_memory())
}

pub struct NatStore(pub Nat);

impl Storable for NatStore {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.0.to_bytes_be().into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bigu = BigUint::from_bytes_be(&bytes);
        Self(Nat(bigu))
    }
}

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
    #[serde(skip, default = "_default_lp")]
    pub liquidity_pools: StableBTreeMap<Principal, NatStore, Memory>,
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
            liquidity_pools: _default_lp(),
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