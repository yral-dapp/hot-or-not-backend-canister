use std::time::SystemTime;

use candid::{CandidType, Deserialize};
use serde::Serialize;

use super::mint_event::MintEvent;

#[derive(Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEvent {
    Mint {
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer,
    Stake,
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
            TokenEvent::Stake => 0,
        }
    }
}
