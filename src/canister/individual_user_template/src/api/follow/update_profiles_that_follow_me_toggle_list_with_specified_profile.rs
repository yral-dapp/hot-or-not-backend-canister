use candid::{CandidType, Deserialize, Principal};
use shared_utils::canister_specific::individual_user_template::types::{
    error::FollowAnotherUserProfileError, follow::FollowEntryDetail,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::update_profiles_i_follow_toggle_list_with_specified_profile::MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST;

#[derive(CandidType, Deserialize)]
pub struct FollowerArg {
    pub follower_principal_id: Principal,
    pub follower_canister_id: Principal,
}

/// # Access Control
/// Only allow calls from canisters of this project
#[ic_cdk::update]
#[candid::candid_method(update)]
async fn update_profiles_that_follow_me_toggle_list_with_specified_profile(
    arg: FollowerArg,
) -> Result<bool, FollowAnotherUserProfileError> {
    let calling_canister_principal = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
            &mut canister_data,
            &calling_canister_principal,
            &arg,
        )
    })
}

fn update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
    canister_data: &mut CanisterData,
    calling_canister_principal: &Principal,
    arg: &FollowerArg,
) -> Result<bool, FollowAnotherUserProfileError> {
    if *calling_canister_principal != arg.follower_canister_id {
        return Err(FollowAnotherUserProfileError::Unauthorized);
    }

    if canister_data.follow_data.follower.len() as u64 > MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST {
        return Err(FollowAnotherUserProfileError::UserITriedToFollowHasTheirFollowersListFull);
    }

    let follow_entry_detail = FollowEntryDetail {
        principal_id: arg.follower_principal_id,
        canister_id: arg.follower_canister_id,
    };

    let follower = &mut canister_data.follow_data.follower;

    if follower.contains(&follow_entry_detail) {
        follower.remove(&follow_entry_detail);
        Ok(false)
    } else {
        follower.add(follow_entry_detail);
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use shared_utils::canister_specific::individual_user_template::types::follow::FollowList;
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_charlie_canister_id,
    };

    use super::*;

    #[test]
    fn test_update_profiles_that_follow_me_toggle_list_with_specified_profile_impl() {
        let mut canister_data = CanisterData::default();
        let mut calling_canister_principal = get_mock_user_charlie_canister_id();
        let arg = FollowerArg {
            follower_principal_id: get_mock_user_alice_principal_id(),
            follower_canister_id: get_mock_user_alice_canister_id(),
        };

        let result = update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
            &mut canister_data,
            &calling_canister_principal,
            &arg,
        );

        assert_eq!(result, Err(FollowAnotherUserProfileError::Unauthorized));

        calling_canister_principal = get_mock_user_alice_canister_id();
        (0..MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST).for_each(|id: u64| {
            let follow_entry_detail = FollowEntryDetail {
                principal_id: Principal::self_authenticating(id.to_ne_bytes()),
                canister_id: Principal::self_authenticating(id.to_ne_bytes()),
            };
            canister_data.follow_data.follower.add(follow_entry_detail);
        });

        let result = update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
            &mut canister_data,
            &calling_canister_principal,
            &arg,
        );

        assert!(result.is_ok());

        let follow_entry_detail = FollowEntryDetail {
            principal_id: Principal::self_authenticating(
                (MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST + 1).to_ne_bytes(),
            ),
            canister_id: Principal::self_authenticating(
                (MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST + 1).to_ne_bytes(),
            ),
        };
        canister_data.follow_data.follower.add(follow_entry_detail);

        let result = update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
            &mut canister_data,
            &calling_canister_principal,
            &arg,
        );

        assert_eq!(
            result,
            Err(FollowAnotherUserProfileError::UserITriedToFollowHasTheirFollowersListFull)
        );

        canister_data.follow_data.follower = FollowList::default();
        let follow_entry_detail = FollowEntryDetail {
            principal_id: arg.follower_principal_id,
            canister_id: arg.follower_canister_id,
        };

        let result = update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
            &mut canister_data,
            &calling_canister_principal,
            &arg,
        );

        assert!(result.is_ok());
        assert!(*result.as_ref().unwrap());
        assert_eq!(canister_data.follow_data.follower.len(), 1_usize);
        assert!(canister_data
            .follow_data
            .follower
            .contains(&follow_entry_detail));

        let result = update_profiles_that_follow_me_toggle_list_with_specified_profile_impl(
            &mut canister_data,
            &calling_canister_principal,
            &arg,
        );

        assert!(result.is_ok());
        assert!(!(*result.as_ref().unwrap()));
        assert_eq!(canister_data.follow_data.follower.len(), 0_usize);
        assert!(!canister_data
            .follow_data
            .follower
            .contains(&follow_entry_detail));
    }
}
