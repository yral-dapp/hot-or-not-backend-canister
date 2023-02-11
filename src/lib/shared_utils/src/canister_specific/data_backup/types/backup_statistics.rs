use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize)]
pub struct BackupStatistics {
    pub number_of_user_entries: u64,
}
