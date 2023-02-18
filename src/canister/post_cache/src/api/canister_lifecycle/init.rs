use shared_utils::common::types::init_args::PostCacheInitArgs;

use crate::{util::access_control, CANISTER_DATA};

#[ic_cdk::init]
#[candid::candid_method(init)]
fn init(init_args: PostCacheInitArgs) {
    // TODO: populate the canister data access control map
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        access_control::setup_initial_access_control_v1(
            &mut canister_data.access_control_map,
            &init_args.known_principal_ids.clone().unwrap_or_default(),
        );

        canister_data.my_known_principal_ids_map =
            init_args.known_principal_ids.unwrap_or_default();
    });
}
