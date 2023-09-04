use candid::{CandidType, Deserialize};

use crate::common::types::known_principal::KnownPrincipalMap;

#[derive(Deserialize, CandidType, Default)]
pub struct ConfigurationInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
    pub signups_enabled: Option<bool>,
}
