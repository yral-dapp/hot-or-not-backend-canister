use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use crate::canister_specific::individual_user_template::types::hot_or_not::{
    BetDirection, BetOutcomeForBetMaker,
};

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEvent {
    Mint {
        amount: u64,
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer,
    Stake {
        amount: u64,
        details: StakeEvent,
        timestamp: SystemTime,
    },
    HotOrNotOutcomePayout {
        amount: u64,
        details: HotOrNotOutcomePayoutEvent,
        timestamp: SystemTime,
    },
}

impl TokenEvent {
    pub fn get_token_amount_for_token_event(self: &Self) -> u64 {
        match self {
            TokenEvent::Mint { details, .. } => match details {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
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
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum StakeEvent {
    BetOnHotOrNotPost {
        post_canister_id: Principal,
        post_id: u64,
        bet_amount: u64,
        bet_direction: BetDirection,
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
        #[serde(default)]
        event_outcome: BetOutcomeForBetMaker,
        #[serde(skip_serializing)]
        winnings_amount: u64,
    },
}

pub const HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE: u64 = 10;
