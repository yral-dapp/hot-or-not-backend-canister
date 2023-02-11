use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum UpdateProfileSetUniqueUsernameError {
    NotAuthorized,
    UsernameAlreadyTaken,
    SendingCanisterDoesNotMatchUserCanisterId,
    UserCanisterEntryDoesNotExist,
    UserIndexCrossCanisterCallFailed,
}

#[derive(CandidType, Debug, Deserialize)]
pub enum GetUserUtilityTokenTransactionHistoryError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}
