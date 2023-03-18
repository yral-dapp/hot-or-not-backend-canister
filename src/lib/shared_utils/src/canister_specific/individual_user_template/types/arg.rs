use candid::{CandidType, Deserialize, Principal};

use crate::common::types::known_principal::KnownPrincipalMap;

use super::hot_or_not::BetDirection;

#[derive(Deserialize, CandidType)]
pub struct IndividualUserTemplateInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
    pub profile_owner: Option<Principal>,
    pub upgrade_version_number: Option<u64>,
}

impl IndividualUserTemplateInitArgs {
    pub fn new(profile_owner: Principal) -> Self {
        Self {
            known_principal_ids: Some(KnownPrincipalMap::default()),
            profile_owner: Some(profile_owner),
            upgrade_version_number: Some(0),
        }
    }
}

#[derive(Deserialize, CandidType, Clone)]
pub struct PlaceBetArg {
    pub post_id: u64,
    pub bet_amount: u64,
    pub bet_direction: BetDirection,
}
