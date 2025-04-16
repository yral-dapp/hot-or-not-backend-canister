use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Default, CandidType, Deserialize, Serialize, Clone)]
pub struct VersionDetails {
    pub version_number: u64, //TODO: remove
    #[serde(default)]
    pub version: String,
}
