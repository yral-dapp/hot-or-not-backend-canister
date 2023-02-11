use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum SetUniqueUsernameError {
    UsernameAlreadyTaken,
    SendingCanisterDoesNotMatchUserCanisterId,
    UserCanisterEntryDoesNotExist,
}
