use crate::{util::cycles::request_cycles_from_subnet_orchestrator, CANISTER_DATA};
use candid::Principal;
use ic_cdk::api::management_canister::main::{
    deposit_cycles, CanisterIdRecord, CanisterInstallMode, InstallCodeArgument,
};

use ic_cdk_macros::update;
use ic_sns_governance::{init::GovernanceCanisterInitPayloadBuilder, pb::v1::governance::Version};

use shared_utils::{
    common::utils::{
        permissions::is_caller_controller_or_global_admin, task::run_task_concurrently,
        upgrade_canister::upgrade_canister_util,
    },
    constant::{
        SNS_TOKEN_ARCHIVE_MODULE_HASH, SNS_TOKEN_GOVERNANCE_MODULE_HASH,
        SNS_TOKEN_INDEX_MODULE_HASH, SNS_TOKEN_LEDGER_MODULE_HASH, SNS_TOKEN_ROOT_MODULE_HASH,
        SNS_TOKEN_SWAP_MODULE_HASH,
    },
};

#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn upgrade_creator_dao_governance_canisters(wasm_module: Vec<u8>) -> Result<(), String> {
    let governance_canisters: Vec<candid::Principal> =
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .cdao_canisters
                .iter()
                .map(|canisters| canisters.governance)
                .collect()
        });

    let recharge_amount = 100_000_000_000;

    let futures = governance_canisters
        .iter()
        .map(|governance_canister_id| async {
            let result = recharge_and_upgrade_canister(
                *governance_canister_id,
                wasm_module.clone(),
                recharge_amount,
            )
            .await;

            match result {
                Ok(()) => Ok(*governance_canister_id),
                Err(e) => Err((*governance_canister_id, e)),
            }
        });

    run_task_concurrently(
        futures,
        10,
        |result| match result {
            Ok(canister_id) => ic_cdk::println!(
                "Governance canister {} upgraded Successfully",
                canister_id.to_text()
            ),
            Err(e) => ic_cdk::println!(
                "Failed upgrading governance Canister {} . Error: {}",
                e.0,
                e.1
            ),
        },
        || false,
    )
    .await;

    Ok(())
}

async fn recharge_and_upgrade_canister(
    canister_id: Principal,
    wasm_module: Vec<u8>,
    recharge_amount: u128,
) -> Result<(), String> {
    //Add cycles in individual canister
    request_cycles_from_subnet_orchestrator(recharge_amount).await?;

    deposit_cycles(CanisterIdRecord { canister_id }, recharge_amount)
        .await
        .map_err(|e| e.1)?;

    let gov_hash = hex::decode(SNS_TOKEN_GOVERNANCE_MODULE_HASH).unwrap();
    let ledger_hash = hex::decode(SNS_TOKEN_LEDGER_MODULE_HASH).unwrap();
    let root_hash = hex::decode(SNS_TOKEN_ROOT_MODULE_HASH).unwrap();
    let swap_hash = hex::decode(SNS_TOKEN_SWAP_MODULE_HASH).unwrap();
    let index_hash = hex::decode(SNS_TOKEN_INDEX_MODULE_HASH).unwrap();
    let archive_hash = hex::decode(SNS_TOKEN_ARCHIVE_MODULE_HASH).unwrap();

    let mut governance_init_payload = GovernanceCanisterInitPayloadBuilder::new().build();

    governance_init_payload.deployed_version = Some(Version {
        governance_wasm_hash: gov_hash,
        ledger_wasm_hash: ledger_hash,
        root_wasm_hash: root_hash,
        swap_wasm_hash: swap_hash,
        archive_wasm_hash: archive_hash,
        index_wasm_hash: index_hash,
    });

    let upgrade_arg = candid::encode_one(governance_init_payload).map_err(|e| e.to_string())?;

    let install_code_argument = InstallCodeArgument {
        mode: CanisterInstallMode::Upgrade(None),
        canister_id,
        wasm_module,
        arg: upgrade_arg,
    };

    upgrade_canister_util(install_code_argument)
        .await
        .map_err(|e| e.1)
}
