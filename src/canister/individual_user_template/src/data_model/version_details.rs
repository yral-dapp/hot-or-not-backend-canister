use candid::{CandidType, Deserialize};
use speedy::{Readable, Writable};

#[derive(Readable, Writable, Default, CandidType, Deserialize)]
pub struct VersionDetails {
    version_number: u64,
}
