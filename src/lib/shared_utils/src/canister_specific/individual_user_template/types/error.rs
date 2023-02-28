use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum GetPostsOfUserProfileError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum GetFollowerOrFollowingError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}
