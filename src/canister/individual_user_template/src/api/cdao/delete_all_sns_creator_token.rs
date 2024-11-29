use ic_cdk::api::management_canister::main::{deposit_cycles, CanisterIdRecord};
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::cdao::DeployedCdaoCanisters,
    common::utils::permissions::is_caller_controller,
};

use crate::CANISTER_DATA;

use super::{
    request_cycles_from_subnet_orchestrator,
    utils::uninstall_code_and_return_empty_canisters_to_subnet_backup_pool, SubnetOrchestrator,
};

#[update(guard = "is_caller_controller")]
pub fn delete_all_creator_token() {
    let deployed_canisters =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.cdao_canisters.clone());

    deployed_canisters
        .iter()
        .for_each(|deployed_cdao_canisters| {
            delete_sns_creator_token(deployed_cdao_canisters);
        });

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.cdao_canisters = vec![];
    })
}

pub fn delete_sns_creator_token(deployed_canisters: &DeployedCdaoCanisters) {
    let canister_ids = deployed_canisters.get_canister_ids();

    const UNINSTALL_RECHARGE_AMOUNT: u128 = 300_000_000_000;

    ic_cdk::spawn(async move {
        let _ = request_cycles_from_subnet_orchestrator(5 * UNINSTALL_RECHARGE_AMOUNT).await;
        for canister_id in canister_ids.iter() {
            let _ = deposit_cycles(
                CanisterIdRecord {
                    canister_id: *canister_id,
                },
                UNINSTALL_RECHARGE_AMOUNT,
            )
            .await;
        }
        uninstall_code_and_return_empty_canisters_to_subnet_backup_pool(canister_ids).await;
    });
}
