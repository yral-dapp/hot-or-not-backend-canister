use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::UpgradeCanisterArg,
    common::{
        types::wasm::{CanisterWasm, WasmType},
        utils::task::run_task_concurrently,
    },
};

use crate::{
    data_model::CanisterUpgradeStatus,
    guard::is_caller::is_caller_global_admin_or_controller,
    utils::{recharge_and_upgrade_subnet_orchestrator, recharge_subnet_orchestrator_if_needed},
    CANISTER_DATA,
};

#[update(guard = "is_caller_global_admin_or_controller")]
pub async fn upgrade_canisters_in_network(
    upgrade_arg: UpgradeCanisterArg,
) -> Result<String, String> {
    match upgrade_arg.canister {
        WasmType::IndividualUserWasm => {
            ic_cdk::spawn(upgrade_individual_canisters(upgrade_arg));
            Ok("Success".into())
        }
        _ => {
            ic_cdk::spawn(upgrade_subnet_canisters(upgrade_arg));
            Ok("Success".into())
        }
    }
}

async fn upgrade_individual_canisters(upgrade_arg: UpgradeCanisterArg) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let canister_wasm = CanisterWasm {
            version: upgrade_arg.version.clone(),
            wasm_blob: upgrade_arg.wasm_blob.clone(),
        };
        canister_data
            .wasms
            .insert(WasmType::IndividualUserWasm, canister_wasm);

        canister_data
            .last_subnet_canister_upgrade_status
            .upgrade_arg = upgrade_arg.clone();
        canister_data.last_subnet_canister_upgrade_status.failures = vec![];
        canister_data.last_subnet_canister_upgrade_status.count = 0;
    });
    let subnet_orchestrator_canisters = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.all_subnet_orchestrator_canisters_list.clone());

    for subnet_orchestrator in subnet_orchestrator_canisters.iter() {
        match recharge_subnet_orchestrator_if_needed(*subnet_orchestrator).await {
            Ok(_) => {}
            Err(e) => {
                CANISTER_DATA.with_borrow_mut(|canister_data| {
                    canister_data
                        .last_subnet_canister_upgrade_status
                        .failures
                        .push((*subnet_orchestrator, e.to_string()));
                });
                continue;
            }
        }
        let res: Result<(String,), String> = ic_cdk::call(
            *subnet_orchestrator,
            "start_upgrades_for_individual_canisters",
            (upgrade_arg.version.clone(), upgrade_arg.wasm_blob.clone()),
        )
        .await
        .map_err(|e| format!("Failed to start upgrades on {}", subnet_orchestrator));

        match res {
            Ok(_) => {}
            Err(e) => CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data
                    .last_subnet_canister_upgrade_status
                    .failures
                    .push((*subnet_orchestrator, e))
            }),
        }
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.last_subnet_canister_upgrade_status.count += 1;
        })
    }
}

async fn upgrade_subnet_canisters(upgrade_arg: UpgradeCanisterArg) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.last_subnet_canister_upgrade_status = CanisterUpgradeStatus {
            upgrade_arg: upgrade_arg.clone(),
            count: 0,
            failures: vec![],
        };
        let canister_wasm = CanisterWasm {
            version: upgrade_arg.version.clone(),
            wasm_blob: upgrade_arg.wasm_blob.clone(),
        };
        canister_data
            .wasms
            .insert(upgrade_arg.canister.clone(), canister_wasm);
    });

    let canister_list = CANISTER_DATA.with_borrow(|canister_data| {
        match upgrade_arg.canister {
            WasmType::SubnetOrchestratorWasm => {
                Ok(canister_data.all_subnet_orchestrator_canisters_list.clone())
            }
            _ => Err(()),
        }
        .unwrap()
    });

    let canister_upgrades = canister_list.iter().map(|canister_id| {
        recharge_and_upgrade_subnet_orchestrator(*canister_id, upgrade_arg.clone())
    });

    let result_callback = |canister_upgrade_result: Result<Principal, (Principal, String)>| {
        match canister_upgrade_result {
            Ok(canister_id) => {}
            Err((canister_id, err)) => {
                ic_cdk::println!("Failed to upgrade canister {canister_id}. Error: {err}");
                CANISTER_DATA.with_borrow_mut(|canister_data| {
                    canister_data
                        .last_subnet_canister_upgrade_status
                        .failures
                        .push((canister_id, err))
                });
            }
        }

        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.last_subnet_canister_upgrade_status.count += 1;
        })
    };

    run_task_concurrently(canister_upgrades, 10, result_callback, || false).await;

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .subnet_canister_upgrade_log
            .append(&canister_data.last_subnet_canister_upgrade_status)
            .expect("Could not write into subnet upgrade log");
    });
}
