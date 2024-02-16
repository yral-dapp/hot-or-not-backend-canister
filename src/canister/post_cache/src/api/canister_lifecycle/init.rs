use ic_cdk_macros::init;
use shared_utils::canister_specific::post_cache::types::arg::PostCacheInitArgs;

use crate::CANISTER_DATA;

#[init]
fn init(init_args: PostCacheInitArgs) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        canister_data.known_principal_ids = init_args.known_principal_ids.unwrap_or_default();
    });
}
