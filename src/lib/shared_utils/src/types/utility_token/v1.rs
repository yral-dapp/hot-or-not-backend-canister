use std::time::SystemTime;

use candid::{CandidType, Deserialize};
use speedy::{Readable, Writable};

use super::v0::MintEvent;

#[derive(Readable, Writable, Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum TokenEventV1 {
    Mint {
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer,
    Stake,
}

impl TokenEventV1 {
    pub fn get_token_amount_for_token_event(self: &Self) -> u64 {
        match self {
            TokenEventV1::Mint { details, .. } => match details {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
            },
            TokenEventV1::Burn => 0,
            TokenEventV1::Transfer => 0,
            TokenEventV1::Stake => 0,
        }
    }
}
