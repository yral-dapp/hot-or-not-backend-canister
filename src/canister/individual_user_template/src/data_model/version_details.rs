use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct VersionDetails {
    version_number: u64,
}
