use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call;
use shared_utils::{common::types::known_principal::KnownPrincipalType, constant};

use crate::CANISTER_DATA;

#[derive(CandidType, Deserialize)]
pub enum AnotherUserFollowedMeError {
    UserIndexCrossCanisterCallFailed,
    UserTryingToFollowMeDoesNotExist,
    NotAuthorized,
    FollowersListFull,
}

// TODO: remove this API in subsequent update
// TODO: implement a separate membership canister that holds entries for all canisters of this project and perform access control
/// # Access Control
/// Only allow calls from canisters of this project
#[deprecated]
#[ic_cdk::update]
#[candid::candid_method(update)]
async fn update_principals_that_follow_me_toggle_list_with_specified_principal(
    user_principal_id_whos_trying_to_follow_me: Principal,
) -> Result<bool, AnotherUserFollowedMeError> {
    let calling_canister_principal = ic_cdk::caller();
    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    let (user_trying_to_follow_me_canister_id,): (Option<Principal>,) = call::call(
        user_index_canister_principal_id,
        "get_user_canister_id_from_user_principal_id",
        (user_principal_id_whos_trying_to_follow_me,),
    )
    .await
    .map_err(|_| AnotherUserFollowedMeError::UserIndexCrossCanisterCallFailed)?;
    let user_trying_to_follow_me_canister_id = user_trying_to_follow_me_canister_id
        .ok_or(AnotherUserFollowedMeError::UserTryingToFollowMeDoesNotExist)?;

    if user_trying_to_follow_me_canister_id != calling_canister_principal {
        return Err(AnotherUserFollowedMeError::NotAuthorized);
    }

    if CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .principals_that_follow_me
            .len()
    }) as u64
        > constant::MAX_USERS_IN_FOLLOWER_FOLLOWING_LIST
    {
        return Err(AnotherUserFollowedMeError::FollowersListFull);
    }

    // * update principals that follow me list
    // my_followers_list.contains(&SPrincipal(user_principal_id_whos_trying_to_follow_me))
    if CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .principals_that_follow_me
            .contains(&user_principal_id_whos_trying_to_follow_me)
    }) {
        CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow_mut()
                .principals_that_follow_me
                .remove(&user_principal_id_whos_trying_to_follow_me);
        });
        Ok(false)
    } else {
        CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow_mut()
                .principals_that_follow_me
                .insert(user_principal_id_whos_trying_to_follow_me);
        });
        Ok(true)
    }
}
