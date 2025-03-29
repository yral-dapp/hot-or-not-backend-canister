use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use crate::canister_specific::individual_user_template::types::{
    hot_or_not::{BetDirection, BetOutcomeForBetMaker},
    pump_n_dump::GameDirection,
};

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEvent {
    Mint {
        amount: u64,
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer {
        amount: u64,
        to_account: Principal,
        timestamp: SystemTime,
    },

    Withdraw {
        amount: u128,
        event_type: WithdrawEvent,
    },

    Receive {
        amount: u64,
        from_account: Principal,
        timestamp: SystemTime,
    },
    Stake {
        amount: u64,
        details: StakeEvent,
        timestamp: SystemTime,
    },

    PumpDumpOutcomePayout {
        amount: u128,
        payout_type: PumpDumpOutcomePayoutEvent,
    },

    HotOrNotOutcomePayout {
        amount: u64,
        details: HotOrNotOutcomePayoutEvent,
        timestamp: SystemTime,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, CandidType)]
pub enum WithdrawEvent {
    WithdrawRequest,
    WithdrawRequestFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub enum PumpDumpOutcomePayoutEvent {
    RewardFromPumpDumpGame {
        game_direction: GameDirection,
        token_root_canister_id: Principal,
    },

    CreatorRewardFromPumpDumpGame,
}

impl TokenEvent {
    pub fn get_token_amount_for_token_event(&self) -> u64 {
        match self {
            TokenEvent::Mint { details, .. } => match details {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
                _ => 0,
            },
            _ => 0,
        }
    }
}

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum MintEvent {
    NewUserSignup {
        new_user_principal_id: Principal,
    },
    Referral {
        referee_user_principal_id: Principal,
        referrer_user_principal_id: Principal,
    },

    Airdrop {
        amount: u64,
    },
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum StakeEvent {
    BetOnHotOrNotPost {
        post_canister_id: Principal,
        post_id: u64,
        bet_amount: u64,
        bet_direction: BetDirection,
    },
    BetFailureRefund {
        bet_amount: u64,
        post_id: u64,
        post_canister_id: Principal,
        bet_direction: BetDirection,
    },
    BetOnPumpDump {
        pumps: u64,
        dumps: u64,
        root_canister_id: Principal,
    },
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum HotOrNotOutcomePayoutEvent {
    CommissionFromHotOrNotBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: u8,
        room_id: u64,
        room_pot_total_amount: u64,
    },
    WinningsEarnedFromBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: u8,
        room_id: u64,
        event_outcome: BetOutcomeForBetMaker,
        winnings_amount: u64,
    },
}

pub const HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE: u64 = 10;
pub const HOT_OR_NOT_BET_WINNINGS_MULTIPLIER: u64 = 2;
