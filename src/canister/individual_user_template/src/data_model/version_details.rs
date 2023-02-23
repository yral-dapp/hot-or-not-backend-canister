use candid::{CandidType, Deserialize};
use serde::Serialize;
use speedy::{Readable, Writable};

#[derive(Readable, Writable, Default, CandidType, Deserialize, Serialize)]
pub struct VersionDetails {
    version_number: u64,
}
