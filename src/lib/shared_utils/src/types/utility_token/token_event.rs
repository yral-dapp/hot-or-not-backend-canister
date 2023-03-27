use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use crate::canister_specific::individual_user_template::types::hot_or_not::BetDirection;

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEvent {
    Mint {
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer,
    Stake {
        details: StakeEvent,
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
            TokenEvent::Burn => 0,
            TokenEvent::Transfer => 0,
            TokenEvent::Stake { .. } => 0,
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
