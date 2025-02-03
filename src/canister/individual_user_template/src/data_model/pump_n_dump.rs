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
    /// Amount that has been obtained from airdrops (lifetime)
    pub net_airdrop: Nat,
    /// user balance
    pub balance: Nat,
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
            net_airdrop: 0u32.into(),
            balance: 0u32.into(),
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
    pub fn add_reward_from_airdrop(&mut self, amount: Nat) {
        self.net_airdrop += amount.clone();
        self.balance += amount;
    }

    pub fn withdrawable_balance(&self) -> Nat {
        if self.net_airdrop >= self.balance {
            0u32.into()
        } else {
            self.balance.clone() - self.net_airdrop.clone()
        }
    }
}