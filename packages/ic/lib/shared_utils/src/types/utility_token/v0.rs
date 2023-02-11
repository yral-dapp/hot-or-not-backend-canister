use candid::{CandidType, Deserialize};
use ic_stable_memory::utils::ic_types::SPrincipal;
use speedy::{Readable, Writable};

#[derive(Readable, Writable, Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum MintEvent {
    NewUserSignup {
        new_user_principal_id: SPrincipal,
    },
    Referral {
        referee_user_principal_id: SPrincipal,
        referrer_user_principal_id: SPrincipal,
    },
}

#[derive(Readable, Writable, Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum TokenEvent {
    Mint(MintEvent),
    Burn,
    Transfer,
    Stake,
}

impl TokenEvent {
    pub fn get_token_amount_for_token_event(self: &Self) -> u64 {
        match self {
            TokenEvent::Mint(mint_event) => match mint_event {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
            },
            TokenEvent::Burn => 0,
            TokenEvent::Transfer => 0,
            TokenEvent::Stake => 0,
        }
    }
}
