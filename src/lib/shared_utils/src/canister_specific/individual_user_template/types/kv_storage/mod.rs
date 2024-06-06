use candid::{CandidType, Principal};

#[derive(CandidType, Copy, Clone)]
pub enum NamespaceErrors {
    Unauthorized,
    NamespaceNotFound,
    UserNotSignedUp,
}

#[derive(CandidType)]
pub struct NamespaceForFrontend {
    pub id: u64,
    pub title: String,
    pub owner_id: Principal,
}
