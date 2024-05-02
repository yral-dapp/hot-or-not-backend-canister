use candid::{CandidType, Principal};
use ic_cdk::api::{
    self,
    call::RejectionCode,
    management_canister::{
        main::{
            self, canister_info, canister_status, start_canister, stop_canister,
            CanisterInfoRequest, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument,
            WasmModule,
        },
        provisional::{CanisterIdRecord, CanisterSettings},
    },
};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    constant::INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
    cycles::calculate_recharge_and_threshold_cycles_for_canister,
};

use crate::CANISTER_DATA;

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
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

pub async fn create_users_canister(
    profile_owner: Option<Principal>,
    version: String,
    individual_user_wasm: Vec<u8>,
) -> Principal {
    let canister_id = create_empty_user_canister().await;
    install_canister_wasm(canister_id, profile_owner, version, individual_user_wasm).await;
    canister_id
}

pub async fn create_empty_user_canister() -> Principal {
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

    canister_id
}

pub async fn install_canister_wasm(
    canister_id: Principal,
    profile_owner: Option<Principal>,
    version: String,
    wasm: Vec<u8>,
) -> Principal {
    let configuration = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.clone());

    let individual_user_tempalate_init_args = IndividualUserTemplateInitArgs {
        profile_owner,
        known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow()
                .configuration
                .known_principal_ids
                .clone()
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
        wasm_module: wasm,
        arg,
    })
    .await
    .unwrap();

    canister_id
}

pub async fn reinstall_canister_wasm(
    canister_id: Principal,
    profile_owner: Option<Principal>,
    version: String,
    wasm: Vec<u8>,
) -> Result<Principal, String> {
    let configuration = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.clone());

    let individual_user_tempalate_init_args = IndividualUserTemplateInitArgs {
        profile_owner,
        known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow()
                .configuration
                .known_principal_ids
                .clone()
        })),
        upgrade_version_number: Some(0),
        version,
        url_to_send_canister_metrics_to: Some(configuration.url_to_send_canister_metrics_to),
    };

    // * encode argument for user canister init lifecycle method
    let arg = match candid::encode_one(individual_user_tempalate_init_args) {
        Ok(arg) => arg,
        Err(err) => return Err(err.to_string()),
    };

    // * install wasm to provisioned canister
    match main::install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Reinstall,
        canister_id,
        wasm_module: wasm,
        arg,
    })
    .await
    {
        Ok(_) => Ok(canister_id),
        Err(err) => Err(err.1),
    }
}

pub async fn upgrade_individual_user_canister(
    canister_id: Principal,
    install_mode: CanisterInstallMode,
    arg: IndividualUserTemplateInitArgs,
    individual_user_wasm: Vec<u8>,
) -> Result<(), (RejectionCode, String)> {
    stop_canister(CanisterIdRecord {
        canister_id: canister_id.clone(),
    })
    .await?;
    let serialized_arg =
        candid::encode_args((arg,)).expect("Failed to serialize the install argument.");

    main::install_code(InstallCodeArgument {
        mode: install_mode,
        canister_id,
        wasm_module: individual_user_wasm,
        arg: serialized_arg,
    })
    .await?;
    start_canister(CanisterIdRecord { canister_id }).await
}

pub async fn recharge_canister_if_below_threshold(canister_id: &Principal) -> Result<(), String> {
    match canister_status(CanisterIdRecord {
        canister_id: *canister_id,
    })
    .await
    {
        Ok((individual_canister_status,)) => {
            let idle_cycles_burned_per_day =
                u128::try_from(individual_canister_status.idle_cycles_burned_per_day.0)
                    .map_err(|e| e.to_string())?;
            let (threshold_balance, recharge_amount) =
                calculate_recharge_and_threshold_cycles_for_canister(
                    idle_cycles_burned_per_day,
                    None,
                );
            let individual_canister_current_balance =
                u128::try_from(individual_canister_status.cycles.0).map_err(|e| e.to_string())?;
            if individual_canister_current_balance < threshold_balance {
                let mut recharge_amount = recharge_amount - individual_canister_current_balance;
                recharge_amount = u128::min(1_000_000_000_000, recharge_amount);
                recharge_canister(canister_id, recharge_amount).await?;
            }

            Ok(())
        }
        Err(e) => {
            recharge_canister(canister_id, INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT).await?;
            Ok(())
        }
    }
}

pub async fn recharge_canister(
    canister_id: &Principal,
    recharge_amount: u128,
) -> Result<(), String> {
    main::deposit_cycles(
        CanisterIdRecord {
            canister_id: *canister_id,
        },
        recharge_amount,
    )
    .await
    .map_err(|e| e.1)
}
