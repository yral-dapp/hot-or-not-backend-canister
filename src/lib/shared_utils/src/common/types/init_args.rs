use candid::{CandidType, Deserialize};

use super::known_principal::KnownPrincipalMapV1;

#[derive(Deserialize, CandidType, Default)]
pub struct PostCacheInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMapV1>,
}
