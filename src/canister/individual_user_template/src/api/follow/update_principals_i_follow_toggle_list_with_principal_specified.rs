use crate::CANISTER_DATA;

use super::update_principals_that_follow_me_toggle_list_with_specified_principal::AnotherUserFollowedMeError;
use candid::{CandidType, Principal};
use ic_cdk::api::call;
use shared_utils::{common::types::known_principal::KnownPrincipalType, constant};

#[derive(CandidType)]
pub enum FollowAnotherUserProfileError {
    NotAuthorized,
    UsersICanFollowListIsFull,
    UserIndexCrossCanisterCallFailed,
    UserToFollowDoesNotExist,
    UserITriedToFollowCrossCanisterCallFailed,
    UserITriedToFollowDidNotFindMe,
    MyCanisterIDDoesNotMatchMyPrincipalCanisterIDMappingSeenByUserITriedToFollow,
    UserITriedToFollowHasTheirFollowersListFull,
}

/// # Access Control
/// Only the user whose profile details are stored in this canister can follow another user's profile.
#[ic_cdk_macros::update]
#[candid::candid_method(update)]
async fn update_principals_i_follow_toggle_list_with_principal_specified(
    user_to_follow: Principal,
) -> Result<bool, FollowAnotherUserProfileError> {
    let current_caller = ic_cdk::caller();

    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);

    if my_principal_id.is_none() {
        return Err(FollowAnotherUserProfileError::NotAuthorized);
    }

    if my_principal_id.unwrap() != current_caller {
        return Err(FollowAnotherUserProfileError::NotAuthorized);
    }

    if CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().principals_i_follow.len())
        as u64
        > constant::MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST
    {
        return Err(FollowAnotherUserProfileError::UsersICanFollowListIsFull);
    }

    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    // * inter canister call to user index to get the user canister id of the user to follow
    let (followee_canister_id,): (Option<Principal>,) = call::call(
        user_index_canister_principal_id,
        "get_user_canister_id_from_user_principal_id",
        (user_to_follow,),
    )
    .await
    .map_err(|_| FollowAnotherUserProfileError::UserIndexCrossCanisterCallFailed)?;
    let followee_canister_id =
        followee_canister_id.ok_or(FollowAnotherUserProfileError::UserToFollowDoesNotExist)?;

    // * inter canister call to update the followee's list of followers
    let (response,): (Result<bool, AnotherUserFollowedMeError>,) = call::call(
        followee_canister_id,
        "update_principals_that_follow_me_toggle_list_with_specified_principal",
        (current_caller,),
    )
    .await
    .map_err(|_| FollowAnotherUserProfileError::UserITriedToFollowCrossCanisterCallFailed)?;

    let following_call_status_inner_bool = response.map_err(|e| match e {
        AnotherUserFollowedMeError::UserIndexCrossCanisterCallFailed => {
            FollowAnotherUserProfileError::UserITriedToFollowCrossCanisterCallFailed
        }
        AnotherUserFollowedMeError::UserTryingToFollowMeDoesNotExist => {
            FollowAnotherUserProfileError::UserITriedToFollowDidNotFindMe
        }
        AnotherUserFollowedMeError::NotAuthorized => {
            FollowAnotherUserProfileError::MyCanisterIDDoesNotMatchMyPrincipalCanisterIDMappingSeenByUserITriedToFollow
        }
        AnotherUserFollowedMeError::FollowersListFull => {
            FollowAnotherUserProfileError::UserITriedToFollowHasTheirFollowersListFull
        }
    })?;

    // * update principals i follow
    if following_call_status_inner_bool {
        CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow_mut()
                .principals_i_follow
                .insert(user_to_follow);
        });
        Ok(true)
    } else {
        CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow_mut()
                .principals_i_follow
                .remove(&user_to_follow);
        });
        Ok(false)
    }
}
