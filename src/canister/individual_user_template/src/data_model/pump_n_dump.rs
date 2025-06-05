use std::borrow::Cow;

use candid::{Nat, Principal};
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cents::CentsToken,
        pump_n_dump::{ParticipatedGameInfo, PumpsAndDumps},
    },
    common::utils::default_pump_dump_onboarding_reward,
};

use super::memory::{get_lp_memory, Memory};

pub fn _default_lp() -> StableBTreeMap<Principal, NatStore, Memory> {
    StableBTreeMap::init(get_lp_memory())
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NatStore(pub Nat);

impl Storable for NatStore {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        self.0 .0.to_bytes_be().into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bigu = BigUint::from_bytes_be(&bytes);
        Self(Nat(bigu))
    }
}

#[derive(Serialize, Deserialize)]
pub struct TokenBetGame {
    pub referral_reward: Nat,
    pub onboarding_reward: Nat,
    pub games: Vec<ParticipatedGameInfo>,
    pub total_dumps: Nat,
    pub total_pumps: Nat,
    #[serde(skip, default = "_default_lp")]
    pub liquidity_pools: StableBTreeMap<Principal, NatStore, Memory>,
    #[serde(default)]
    pub cents: CentsToken,
}

impl Default for TokenBetGame {
    fn default() -> Self {
        Self {
            onboarding_reward: default_pump_dump_onboarding_reward(),
            referral_reward: (1e9 as u64).into(),
            liquidity_pools: _default_lp(),
            games: vec![],
            total_pumps: 0u32.into(),
            total_dumps: 0u32.into(),
            cents: Default::default(),
        }
    }
}

impl TokenBetGame {
    pub fn get_pumps_dumps(&self) -> PumpsAndDumps {
        PumpsAndDumps {
            pumps: self.total_pumps.clone(),
            dumps: self.total_dumps.clone(),
        }
    }
}
