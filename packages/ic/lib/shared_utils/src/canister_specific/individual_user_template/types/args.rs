use candid::{CandidType, Deserialize, Principal};

use crate::common::types::known_principal::KnownPrincipalMapV1;

#[derive(Deserialize, CandidType)]
pub struct IndividualUserTemplateInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMapV1>,
    pub profile_owner: Option<Principal>,
}

impl IndividualUserTemplateInitArgs {
    pub fn new(profile_owner: Principal) -> Self {
        Self {
            known_principal_ids: Some(KnownPrincipalMapV1::default()),
            profile_owner: Some(profile_owner),
        }
    }
}
