use std::borrow::Cow;

use candid::{Nat, Principal};
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        pump_n_dump::{ParticipatedGameInfo, PumpsAndDumps},
        token::{get_earnings_amount_from_winnings_amount, TokenTransactions},
    },
    common::{
        types::utility_token::token_event::{
            HotOrNotOutcomePayoutEvent, MintEvent, PumpDumpOutcomePayoutEvent, StakeEvent,
            TokenEvent, WithdrawEvent, HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE,
        },
        utils::default_pump_dump_onboarding_reward,
    },
    constant::GDOLLR_TO_E8S,
};

use super::memory::{get_lp_memory, Memory};

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

impl PumpAndDumpGame {
    pub fn withdrawable_balance(&self) -> Nat {
        if self.net_airdrop >= self.balance {
            0_u32.into()
        } else {
            self.balance.clone() - self.net_airdrop.clone()
        }
    }

    pub fn get_net_earnings(&self) -> Nat {
        self.net_earnings.clone()
    }

    pub fn get_net_airdrop(&self) -> Nat {
        self.net_airdrop.clone()
    }

    pub fn get_pumps_dumps(&self) -> PumpsAndDumps {
        PumpsAndDumps {
            pumps: self.total_pumps.clone(),
            dumps: self.total_dumps.clone(),
        }
    }
}

impl TokenTransactions for PumpAndDumpGame {
    fn get_current_token_balance(&self) -> u128 {
        self.balance.clone().0.try_into().unwrap()
    }

    fn handle_token_event(&mut self, token_event: TokenEvent) {
        match token_event {
            TokenEvent::Mint {
                details, amount, ..
            } => match details {
                MintEvent::NewUserSignup { .. } => {
                    self.balance += amount as u128;
                    self.net_airdrop += amount as u128;
                    self.net_earnings += amount as u128;
                }
                MintEvent::Referral { .. } => {
                    self.net_airdrop += amount as u128;
                    self.balance += amount as u128;
                    self.net_earnings += amount as u128;
                }
                MintEvent::Airdrop { amount } => {
                    self.net_airdrop += amount as u128;
                    self.balance += amount as u128;
                    self.net_earnings += amount as u128;
                }
            },
            TokenEvent::Burn => {}
            TokenEvent::Transfer { amount, .. } => {
                self.balance -= amount as u128;
            }
            TokenEvent::Receive { amount, .. } => {
                self.balance += amount as u128;
            }
            TokenEvent::Stake {
                details, amount, ..
            } => match details {
                StakeEvent::BetOnHotOrNotPost { bet_amount, .. } => {
                    self.balance -= bet_amount as u128;
                }
                StakeEvent::BetFailureRefund { bet_amount, .. } => {
                    self.balance += bet_amount as u128;
                }
                StakeEvent::BetOnPumpDump {
                    pumps,
                    dumps,
                    root_canister_id,
                } => {
                    self.balance -= amount as u128;
                    self.total_pumps += pumps as u128;
                    self.total_dumps += dumps as u128;
                }
            },
            TokenEvent::HotOrNotOutcomePayout { details, .. } => match details {
                HotOrNotOutcomePayoutEvent::CommissionFromHotOrNotBet {
                    room_pot_total_amount,
                    ..
                } => {
                    self.balance += (room_pot_total_amount
                        * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
                        / 100) as u128;
                    self.net_earnings += (room_pot_total_amount
                        * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
                        / 100) as u128;
                }
                HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                    winnings_amount, ..
                } => {
                    self.balance += winnings_amount as u128;
                    self.net_earnings +=
                        get_earnings_amount_from_winnings_amount(&winnings_amount) as u128;
                }
            },
            TokenEvent::Withdraw { amount, event_type } => match event_type {
                WithdrawEvent::WithdrawRequest => {
                    self.balance -= amount;
                }
                WithdrawEvent::WithdrawRequestFailed => {
                    self.balance += amount;
                }
            },
            TokenEvent::PumpDumpOutcomePayout {
                amount,
                payout_type,
            } => match payout_type {
                PumpDumpOutcomePayoutEvent::RewardFromPumpDumpGame { .. } => {
                    self.balance += amount;
                    self.net_earnings += amount;
                }
                PumpDumpOutcomePayoutEvent::CreatorRewardFromPumpDumpGame => {
                    self.balance += amount;
                    self.net_earnings += amount;
                }
            },
        }
    }
}
