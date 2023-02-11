use candid::Principal;
use ic_cdk::api::management_canister::{
    main::{self, CanisterStatusResponse},
    provisional::CanisterIdRecord,
};
use shared_utils::access_control::{self, UserAccessRole};

use crate::CANISTER_DATA;

// TODO: move this to the individual canisters
// TODO: Do this by calling this via the user_index canister
// TODO: Also investigate why global principal is unable to call this. Are we not setting global principal as a controller when provisioning this canister?
// TODO: Remove this endpoint altogether if the testing runtime has direct access to this data
#[ic_cdk_macros::update]
#[candid::candid_method(update)]
async fn get_canister_status_from_management_canister(
    canister_id: Principal,
) -> CanisterStatusResponse {
    let api_caller = ic_cdk::caller();

    let access_control_map = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().access_control_map.clone());

    // TODO: update the return type of this method so that unauthorized callers are informed accordingly
    if !access_control::does_principal_have_role_v2(
        &access_control_map,
        UserAccessRole::CanisterAdmin,
        api_caller,
    ) {
        panic!("Unauthorized caller");
    };

    let (response,): (CanisterStatusResponse,) =
        main::canister_status(CanisterIdRecord { canister_id })
            .await
            .unwrap();

    response
}
