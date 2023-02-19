use candid::{CandidType, Deserialize, Principal};

use crate::common::types::known_principal::KnownPrincipalMap;

#[derive(Deserialize, CandidType)]
pub struct IndividualUserTemplateInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
    pub profile_owner: Option<Principal>,
}

impl IndividualUserTemplateInitArgs {
    pub fn new(profile_owner: Principal) -> Self {
        Self {
            known_principal_ids: Some(KnownPrincipalMap::default()),
            profile_owner: Some(profile_owner),
        }
    }
}
