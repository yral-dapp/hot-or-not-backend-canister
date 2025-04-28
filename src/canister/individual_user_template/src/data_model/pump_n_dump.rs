use std::{borrow::Cow, collections::BTreeMap, time::SystemTime};

use candid::{Nat, Principal};
use ic_cdk::api::management_canister::main::CanisterId;
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        cents::CentsToken,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{
            BetDetails, BetDirection, BettingStatus, GlobalBetId, GlobalRoomId, PlacedBetDetail,
            RoomDetailsV1, SlotDetailsV1, SlotId, StablePrincipal,
        },
        pump_n_dump::{ParticipatedGameInfo, ParticipatedGameInfoV0, PumpsAndDumps},
    },
    common::{types::app_primitive_type::PostId, utils::default_pump_dump_onboarding_reward},
};

use crate::CANISTER_DATA;

use super::{
    memory::{
        get_bet_details_memory_v2, get_lp_memory, get_post_principal_memory_v2,
        get_room_details_memory_v2, get_slot_details_memory_v2, Memory,
    },
    CanisterData,
};

pub fn _default_lp() -> StableBTreeMap<Principal, NatStore, Memory> {
    StableBTreeMap::init(get_lp_memory())
}

#[derive(Serialize, Deserialize, Clone)]
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
pub struct HotOrNotGameDetails {
    #[serde(skip, default = "_default_room_details_v2")]
    pub room_details_map:
        ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory>,
    #[serde(skip, default = "_default_slot_details_map_v2")]
    pub slot_details_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory>,
    #[serde(skip, default = "_default_post_principal_map_v2")]
    pub post_principal_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>,
    #[serde(skip, default = "_default_bet_details_v2")]
    pub bet_details_map: ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory>,
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
}

impl Default for HotOrNotGameDetails {
    fn default() -> Self {
        Self {
            room_details_map: _default_room_details_v2(),
            slot_details_map: _default_slot_details_map_v2(),
            post_principal_map: _default_post_principal_map_v2(),
            bet_details_map: _default_bet_details_v2(),
            all_hot_or_not_bets_placed: BTreeMap::default(),
        }
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
    pub hot_or_not_bet_details: HotOrNotGameDetails,
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
            hot_or_not_bet_details: Default::default(),
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

pub fn _default_room_details_v2(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory_v2())
}

pub fn _default_bet_details_v2(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory_v2())
}

pub fn _default_post_principal_map_v2(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory_v2())
}

pub fn _default_slot_details_map_v2(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, SlotId), SlotDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_slot_details_memory_v2())
}
