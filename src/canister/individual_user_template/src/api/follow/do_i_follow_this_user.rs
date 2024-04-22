use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::{
    arg::FolloweeArg, error::FollowAnotherUserProfileError, follow::FollowEntryDetail,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};

#[query]
fn do_i_follow_this_user(arg: FolloweeArg) -> Result<bool, FollowAnotherUserProfileError> {
    let current_caller = ic_cdk::caller();

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();
        do_i_follow_this_user_impl(&canister_data, &arg, &current_caller)
    })
}

fn do_i_follow_this_user_impl(
    canister_data: &CanisterData,
    arg: &FolloweeArg,
    current_caller: &Principal,
) -> Result<bool, FollowAnotherUserProfileError> {
    if *current_caller == Principal::anonymous() {
        return Err(FollowAnotherUserProfileError::Unauthenticated);
    }

    let my_principal_id = canister_data
        .profile
        .principal_id
        .expect("My principal ID not set");

    if my_principal_id != *current_caller {
        return Err(FollowAnotherUserProfileError::Unauthorized);
    }

    let follow_entry_detail = FollowEntryDetail {
        principal_id: arg.followee_principal_id,
        canister_id: arg.followee_canister_id,
    };

    Ok(canister_data
        .follow_data
        .following
        .contains(&follow_entry_detail))
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_canister_id,
        get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    };

    use super::*;

    #[test]
    fn test_do_i_follow_this_user_impl() {
        let mut canister_data = CanisterData::default();
        let arg = FolloweeArg {
            followee_principal_id: get_mock_user_bob_principal_id(),
            followee_canister_id: get_mock_user_bob_canister_id(),
        };

        let current_caller = Principal::anonymous();

        let result = do_i_follow_this_user_impl(&canister_data, &arg, &current_caller);

        assert_eq!(result, Err(FollowAnotherUserProfileError::Unauthenticated));

        canister_data.profile.principal_id = Some(get_mock_user_alice_principal_id());

        let current_caller = get_mock_user_charlie_principal_id();

        let result = do_i_follow_this_user_impl(&canister_data, &arg, &current_caller);

        assert_eq!(result, Err(FollowAnotherUserProfileError::Unauthorized));

        let current_caller = get_mock_user_alice_principal_id();

        let result = do_i_follow_this_user_impl(&canister_data, &arg, &current_caller);

        assert_eq!(result, Ok(false));

        canister_data.follow_data.following.add(FollowEntryDetail {
            principal_id: get_mock_user_bob_principal_id(),
            canister_id: get_mock_user_bob_canister_id(),
        });

        let result = do_i_follow_this_user_impl(&canister_data, &arg, &current_caller);

        assert_eq!(result, Ok(true));
    }
}
