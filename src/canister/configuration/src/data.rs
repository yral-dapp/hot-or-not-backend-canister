use candid::{CandidType, Deserialize};
use shared_utils::common::types::known_principal::KnownPrincipalMap;

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    pub known_principal_ids: KnownPrincipalMap,
    pub signups_enabled: bool,
}
