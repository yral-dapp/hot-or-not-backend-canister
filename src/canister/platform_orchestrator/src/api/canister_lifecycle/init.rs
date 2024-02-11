use candid::Principal;
use shared_utils::canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs;
use crate::CANISTER_DATA;




#[ic_cdk::init]
#[candid::candid_method(init)]
fn init(init_args: PlatformOrchestratorInitArgs) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version_detail.version = init_args.version;
        canister_data.all_subnet_orchestrator_canisters_list.insert(Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap());
        canister_data.subet_orchestrator_with_capacity_left.insert(Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap());
        canister_data.all_post_cache_orchestrator_list.insert(Principal::from_text("y6yjf-jyaaa-aaaal-qbd6q-cai").unwrap());
    })
}