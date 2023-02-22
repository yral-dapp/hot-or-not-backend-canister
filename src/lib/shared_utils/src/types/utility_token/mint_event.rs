use candid::{CandidType, Deserialize, Principal};
use ic_stable_memory::utils::ic_types::SPrincipal;

#[derive(Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum MintEvent {
    NewUserSignup {
        new_user_principal_id: SPrincipal,
    },
    Referral {
        referee_user_principal_id: SPrincipal,
        referrer_user_principal_id: SPrincipal,
    },
}

#[derive(Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum MintEventV1 {
    NewUserSignup {
        new_user_principal_id: Principal,
    },
    Referral {
        referee_user_principal_id: Principal,
        referrer_user_principal_id: Principal,
    },
}

impl From<MintEvent> for MintEventV1 {
    fn from(value: MintEvent) -> Self {
        match value {
            MintEvent::NewUserSignup {
                new_user_principal_id,
            } => MintEventV1::NewUserSignup {
                new_user_principal_id: new_user_principal_id.0,
            },
            MintEvent::Referral {
                referee_user_principal_id,
                referrer_user_principal_id,
            } => MintEventV1::Referral {
                referee_user_principal_id: referee_user_principal_id.0,
                referrer_user_principal_id: referrer_user_principal_id.0,
            },
        }
    }
}
