use shared_utils::canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs;

use crate::CANISTER_DATA;



#[ic_cdk::init]
#[candid::candid_method(init)]
fn init(init_args: PlatformOrchestratorInitArgs) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell.borrow_mut().version_detail.version = init_args.version;
    })
}