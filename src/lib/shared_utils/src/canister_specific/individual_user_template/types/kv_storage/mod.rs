use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum NamespaceErrors {
    Unauthorized,
    NamespaceNotFound,
    UserNotSignedUp,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct NamespaceForFrontend {
    pub id: u64,
    pub title: String,
    pub owner_id: Principal,
}
