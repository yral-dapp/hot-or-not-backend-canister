use candid::{CandidType, Deserialize};
use serde::Serialize;
use shared_utils::common::types::known_principal::KnownPrincipalMap;

#[derive(Default, Deserialize, CandidType, Serialize, Clone)]
pub struct Configuration {
    pub known_principal_ids: KnownPrincipalMap,
    pub signups_open_on_this_subnet: bool,
    #[serde(default)]
    pub url_to_send_canister_metrics_to: String,
}
