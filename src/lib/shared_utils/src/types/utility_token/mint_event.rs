use candid::{CandidType, Deserialize, Principal};
use ic_stable_memory::utils::ic_types::SPrincipal;
use serde::{Deserializer, Serialize};

#[derive(Clone, Copy, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum MintEvent {
    NewUserSignup {
        #[serde(deserialize_with = "principal_deserializer")]
        new_user_principal_id: Principal,
    },
    Referral {
        #[serde(deserialize_with = "principal_deserializer")]
        referee_user_principal_id: Principal,
        #[serde(deserialize_with = "principal_deserializer")]
        referrer_user_principal_id: Principal,
    },
}

fn principal_deserializer<'de, D>(deserializer: D) -> Result<Principal, D::Error>
where
    D: Deserializer<'de>,
{
    let previous = SPrincipal::deserialize(deserializer)?;

    Ok(previous.0)
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
                new_user_principal_id: new_user_principal_id,
            },
            MintEvent::Referral {
                referee_user_principal_id,
                referrer_user_principal_id,
            } => MintEventV1::Referral {
                referee_user_principal_id: referee_user_principal_id,
                referrer_user_principal_id: referrer_user_principal_id,
            },
        }
    }
}
