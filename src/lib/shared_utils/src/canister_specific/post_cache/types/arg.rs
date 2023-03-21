use candid::{CandidType, Deserialize};

use crate::common::types::known_principal::KnownPrincipalMap;

#[derive(Deserialize, CandidType, Default)]
pub struct PostCacheInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
}
