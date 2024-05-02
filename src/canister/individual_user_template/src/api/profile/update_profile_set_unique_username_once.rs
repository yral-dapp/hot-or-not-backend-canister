use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};
use ic_cdk::api::call;
use ic_cdk_macros::update;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    types::canister_specific::{
        individual_user_template::error_types::UpdateProfileSetUniqueUsernameError,
        user_index::error_types::SetUniqueUsernameError,
    },
};

/// # Access Control
/// Only the user whose profile details are stored in this canister can update their details.
#[update]
async fn update_profile_set_unique_username_once(
    new_unique_username: String,
) -> Result<(), UpdateProfileSetUniqueUsernameError> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id)
        .unwrap();

    if current_caller != my_principal_id {
        return Err(UpdateProfileSetUniqueUsernameError::NotAuthorized);
    }

    update_last_canister_functionality_access_time();

    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    // * cross canister call
    let (response,): (Result<(), SetUniqueUsernameError>,) = call::call(
        user_index_canister_principal_id,
        "update_index_with_unique_user_name_corresponding_to_user_principal_id",
        (new_unique_username.clone(), current_caller),
    )
    .await
    .map_err(|_| UpdateProfileSetUniqueUsernameError::UserIndexCrossCanisterCallFailed)?;

    match response {
        Ok(()) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                let mut profile = canister_data_ref_cell.borrow_mut().profile.clone();
                profile.unique_user_name = Some(new_unique_username);
                canister_data_ref_cell.borrow_mut().profile = profile;
            });
            Ok(())
        }
        Err(SetUniqueUsernameError::UsernameAlreadyTaken) => {
            Err(UpdateProfileSetUniqueUsernameError::UsernameAlreadyTaken)
        }
        Err(SetUniqueUsernameError::SendingCanisterDoesNotMatchUserCanisterId) => {
            Err(UpdateProfileSetUniqueUsernameError::SendingCanisterDoesNotMatchUserCanisterId)
        }
        Err(SetUniqueUsernameError::UserCanisterEntryDoesNotExist) => {
            Err(UpdateProfileSetUniqueUsernameError::UserCanisterEntryDoesNotExist)
        }
    }
}
