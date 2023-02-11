use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub enum TopPostsFetchError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}
