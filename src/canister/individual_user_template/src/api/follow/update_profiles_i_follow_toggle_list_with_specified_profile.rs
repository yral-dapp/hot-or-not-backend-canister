use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, util::cycles::notify_to_recharge_canister, CANISTER_DATA,
};

use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::{
    arg::FolloweeArg, error::FollowAnotherUserProfileError, follow::FollowEntryDetail,
};

use super::update_profiles_that_follow_me_toggle_list_with_specified_profile::FollowerArg;

pub const MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST: u64 = 10_000;

/// # Access Control
/// Only the user whose profile details are stored in this canister can follow another user's profile.
#[update]
async fn update_profiles_i_follow_toggle_list_with_specified_profile(
    arg: FolloweeArg,
) -> Result<bool, FollowAnotherUserProfileError> {
    notify_to_recharge_canister();

    let current_caller = ic_cdk::caller();

    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id)
        .expect("Principal Id should be set");

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();

        validate_incoming_request(&canister_data, &current_caller, &my_principal_id)
    })?;

    let my_canister_id = ic_cdk::id();

    // * inter canister call to update the followee's list of followers
    let follow_status = ic_cdk::call::<_, (Result<bool, FollowAnotherUserProfileError>,)>(
        arg.followee_canister_id,
        "update_profiles_that_follow_me_toggle_list_with_specified_profile",
        (FollowerArg {
            follower_principal_id: my_principal_id,
            follower_canister_id: my_canister_id,
        },),
    )
    .await
    .map_err(|_| FollowAnotherUserProfileError::UserITriedToFollowCrossCanisterCallFailed)?
    .0?;

    let followee_entry_detail = FollowEntryDetail {
        principal_id: arg.followee_principal_id,
        canister_id: arg.followee_canister_id,
    };

    CANISTER_DATA.with(|canister_data_ref_cell| {
        add_or_remove_followee_depending_on_follow_status(
            &mut canister_data_ref_cell.borrow_mut(),
            &follow_status,
            &followee_entry_detail,
        )
    })?;

    Ok(follow_status)
}

fn validate_incoming_request(
    canister_data: &CanisterData,
    current_caller: &Principal,
    my_principal_id: &Principal,
) -> Result<(), FollowAnotherUserProfileError> {
    if *current_caller == Principal::anonymous() {
        return Err(FollowAnotherUserProfileError::Unauthenticated);
    }

    if *my_principal_id != *current_caller {
        return Err(FollowAnotherUserProfileError::Unauthorized);
    }

    if canister_data.follow_data.following.len() as u64 > MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST {
        return Err(FollowAnotherUserProfileError::UsersICanFollowListIsFull);
    }

    Ok(())
}

fn add_or_remove_followee_depending_on_follow_status(
    canister_data: &mut CanisterData,
    follow_status: &bool,
    followee_entry_detail: &FollowEntryDetail,
) -> Result<(), FollowAnotherUserProfileError> {
    let following = &mut canister_data.follow_data.following;

    if *follow_status {
        following.add(followee_entry_detail.clone());
    } else {
        following.remove(followee_entry_detail);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_validate_incoming_request() {
        let mut canister_data = CanisterData::default();
        let mut current_caller = Principal::anonymous();
        let my_principal_id = get_mock_user_alice_principal_id();

        let result = validate_incoming_request(&canister_data, &current_caller, &my_principal_id);

        assert_eq!(result, Err(FollowAnotherUserProfileError::Unauthenticated));

        current_caller = get_mock_user_bob_principal_id();

        let result = validate_incoming_request(&canister_data, &current_caller, &my_principal_id);

        assert_eq!(result, Err(FollowAnotherUserProfileError::Unauthorized));

        current_caller = get_mock_user_alice_principal_id();

        (0..MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST).for_each(|id: u64| {
            let follow_entry_detail = FollowEntryDetail {
                principal_id: Principal::self_authenticating(id.to_ne_bytes()),
                canister_id: Principal::self_authenticating(id.to_ne_bytes()),
            };
            canister_data.follow_data.following.add(follow_entry_detail);
        });

        let result = validate_incoming_request(&canister_data, &current_caller, &my_principal_id);

        assert!(result.is_ok());

        let follow_entry_detail = FollowEntryDetail {
            principal_id: Principal::self_authenticating(
                (MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST + 1).to_ne_bytes(),
            ),
            canister_id: Principal::self_authenticating(
                (MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST + 1).to_ne_bytes(),
            ),
        };
        canister_data.follow_data.following.add(follow_entry_detail);

        let result = validate_incoming_request(&canister_data, &current_caller, &my_principal_id);

        assert_eq!(
            result,
            Err(FollowAnotherUserProfileError::UsersICanFollowListIsFull)
        );
    }

    #[test]
    fn test_add_or_remove_followee_depending_on_follow_status() {
        let mut canister_data = CanisterData::default();
        let follow_status = true;
        let followee_entry_detail = FollowEntryDetail {
            principal_id: get_mock_user_alice_principal_id(),
            canister_id: get_mock_user_alice_canister_id(),
        };

        let result = add_or_remove_followee_depending_on_follow_status(
            &mut canister_data,
            &follow_status,
            &followee_entry_detail,
        );

        assert!(result.is_ok());
        assert_eq!(canister_data.follow_data.following.len(), 1);

        let follow_status = false;

        let result = add_or_remove_followee_depending_on_follow_status(
            &mut canister_data,
            &follow_status,
            &followee_entry_detail,
        );

        assert!(result.is_ok());
        assert_eq!(canister_data.follow_data.following.len(), 0);
    }
}
