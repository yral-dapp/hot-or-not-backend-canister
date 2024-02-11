use candid::{Principal, CandidType};
use ic_cdk::api::{
    self,
    call::RejectionCode,
    management_canister::{
        main::{self, CanisterInstallMode, CreateCanisterArgument, WasmModule, InstallCodeArgument, stop_canister, start_canister},
        provisional::{CanisterSettings, CanisterIdRecord},
    },
};
use serde::{Serialize, Deserialize};
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    constant::{INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT, CYCLES_THRESHOLD_TO_INITIATE_RECHARGE},
};

use crate::CANISTER_DATA;

#[derive( CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct CustomInstallCodeArgument {
    /// See [CanisterInstallMode].
    pub mode: CanisterInstallMode,
    /// Principle of the canister.
    pub canister_id: Principal,
    /// Code to be installed.
    pub wasm_module: WasmModule,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    pub arg: Vec<u8>,
    /// sender_canister_version must be set to ic_cdk::api::canister_version()
    pub sender_canister_version: Option<u64>,
    /// drop stable memory after install/upgrade execution.
    pub unsafe_drop_stable_memory: Option<bool>,
}

pub async fn create_users_canister(profile_owner: Option<Principal>, version: String, individual_user_wasm: Vec<u8>) -> Principal {
    // * config for provisioning canister
    let arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![
                // * this subnet_orchestrator canister
                api::id(),
            ]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };

    // * provisioned canister
    let canister_id: Principal =
        main::create_canister(arg, INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT)
            .await
            .unwrap()
            .0
            .canister_id;

    let configuration = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.clone());

    let individual_user_tempalate_init_args = IndividualUserTemplateInitArgs {
        profile_owner: profile_owner,
        known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell.borrow().configuration.known_principal_ids.clone()
        })),
        upgrade_version_number: Some(0),
        version,
        url_to_send_canister_metrics_to: Some(configuration.url_to_send_canister_metrics_to),
    };

    // * encode argument for user canister init lifecycle method
    let arg = candid::encode_one(individual_user_tempalate_init_args)
        .expect("Failed to serialize the install argument.");

    // * install wasm to provisioned canister
    main::install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: individual_user_wasm,
        arg,
    })
    .await
    .unwrap();

    canister_id
}

pub async fn upgrade_individual_user_canister(
    canister_id: Principal,
    install_mode: CanisterInstallMode,
    arg: IndividualUserTemplateInitArgs,
    individual_user_wasm: Vec<u8>
) -> Result<(), (RejectionCode, String)> {
    stop_canister(CanisterIdRecord {canister_id: canister_id.clone()}).await?;
    let serialized_arg =
        candid::encode_args((arg,)).expect("Failed to serialize the install argument.");

        main::install_code(InstallCodeArgument {
            mode: install_mode,
            canister_id,
            wasm_module: individual_user_wasm,
            arg: serialized_arg,
        })
        .await?;
    start_canister(CanisterIdRecord {canister_id}).await
}

pub async fn recharge_canister_if_below_threshold(canister_id: &Principal) -> Result<(), String> {

    let is_canister_below_threshold_balance = is_canister_below_threshold_balance(&canister_id).await;

    if is_canister_below_threshold_balance {
        recharge_canister(canister_id).await?;    
    }

    Ok(())
}


pub async fn is_canister_below_threshold_balance(canister_id: &Principal) -> bool {
    let response: Result<(u128,), (_, _)> =
        ic_cdk::call(*canister_id, "get_user_caniser_cycle_balance", ()).await;

    if response.is_err() {
        return true;
    }

    let (balance,): (u128,) = response.unwrap();

    if balance < CYCLES_THRESHOLD_TO_INITIATE_RECHARGE {
        return true;
    }

    false
}

pub async fn recharge_canister(canister_id: &Principal) -> Result<(), String> {
    main::deposit_cycles(
        CanisterIdRecord {
            canister_id: *canister_id,
        },
        INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
    )
    .await
    .map_err(|e| e.1)
}