use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;

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

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum GetFollowerOrFollowingPageError {
    Unauthenticated,
    Unauthorized,
}

#[derive(CandidType, PartialEq, Eq, Debug, Deserialize)]
pub enum BetOnCurrentlyViewingPostError {
    BettingClosed,
    InsufficientBalance,
    Unauthorized,
    UserAlreadyParticipatedInThisPost,
    UserNotLoggedIn,
    UserPrincipalNotSet,
    PostCreatorCanisterCallFailed,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum FollowAnotherUserProfileError {
    Unauthenticated,
    Unauthorized,
    UsersICanFollowListIsFull,
    UserITriedToFollowCrossCanisterCallFailed,
    UserITriedToFollowHasTheirFollowersListFull,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum CdaoDeployError {
    Unregistered,
    TokenLimit(usize),
    CallError(RejectionCode, String),
    InvalidInitPayload(String),
}

impl From<(RejectionCode, String)> for CdaoDeployError {
    fn from((rejection_code, error_message): (RejectionCode, String)) -> Self {
        CdaoDeployError::CallError(rejection_code, error_message)
    }
}
