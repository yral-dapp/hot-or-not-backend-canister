use ic_cdk_macros::init;
use shared_utils::canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs;
use crate::CANISTER_DATA;




#[init]
fn init(init_args: PlatformOrchestratorInitArgs) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version_detail.version = init_args.version;
    })
}