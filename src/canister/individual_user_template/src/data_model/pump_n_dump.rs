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
        pump_n_dump::{ParticipatedGameInfo, PumpsAndDumps},
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

#[derive(Serialize, Deserialize)]
#[serde(from = "PumpAndDumpGame")]
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

impl From<PumpAndDumpGame> for TokenBetGame {
    fn from(pump_and_dump_game: PumpAndDumpGame) -> Self {
        Self {
            referral_reward: pump_and_dump_game.referral_reward,
            onboarding_reward: pump_and_dump_game.onboarding_reward,
            games: pump_and_dump_game.games,
            total_pumps: pump_and_dump_game.total_pumps,
            total_dumps: pump_and_dump_game.total_dumps,
            liquidity_pools: pump_and_dump_game.liquidity_pools,
            hot_or_not_bet_details: Default::default(),
            cents: CentsToken::default(),
        }
    }
}

impl Default for PumpAndDumpGame {
    fn default() -> Self {
        Self {
            net_airdrop: 0u32.into(),
            balance: 0u32.into(),
            // 1000 gDOLLR
            referral_reward: (1e9 as u64).into(),
            onboarding_reward: default_pump_dump_onboarding_reward(),
            liquidity_pools: _default_lp(),
            games: vec![],
            total_pumps: 0u32.into(),
            total_dumps: 0u32.into(),
            net_earnings: 0u32.into(),
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

impl TokenBetGame {
    pub fn register_hot_or_not_bet_for_post_v1(
        &mut self,
        post_id: u64,
        bet_maker_principal_id: Principal,
        bet_maker_canister_id: Principal,
        place_bet_arg: &PlaceBetArg,
        current_time_when_request_being_made: &SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let post =
                CanisterData::get_post_mut(&mut canister_data.all_created_posts, post_id).unwrap();
            let PlaceBetArg {
                bet_amount,
                bet_direction,
                ..
            } = place_bet_arg;
            let betting_status = post.place_hot_or_not_bet_v1(
                &bet_maker_principal_id,
                &bet_maker_canister_id,
                *bet_amount,
                bet_direction,
                current_time_when_request_being_made,
                &mut self.hot_or_not_bet_details.room_details_map,
                &mut self.hot_or_not_bet_details.bet_details_map,
                &mut self.hot_or_not_bet_details.post_principal_map,
                &mut self.hot_or_not_bet_details.slot_details_map,
            )?;

            match *bet_direction {
                BetDirection::Hot => {
                    canister_data.profile.profile_stats.hot_bets_received += 1;
                }
                BetDirection::Not => {
                    canister_data.profile.profile_stats.not_bets_received += 1;
                }
            }

            Ok(betting_status)
        })
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
