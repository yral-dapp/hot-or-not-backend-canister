use candid::{Principal, CandidType};
use ic_cdk::api::{
    self,
    call::RejectionCode,
    management_canister::{
        main::{self, CanisterInstallMode, CreateCanisterArgument, WasmModule, InstallCodeArgument, stop_canister, start_canister, canister_status, CanisterStatusType},
        provisional::{CanisterSettings, CanisterIdRecord},
    }, canister_version,
};
use serde::{Serialize, Deserialize};
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    constant::INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
};

use crate::CANISTER_DATA;

const INDIVIDUAL_USER_TEMPLATE_CANISTER_WASM: &[u8] = include_bytes!(
    "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
);

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

pub async fn create_users_canister(profile_owner: Principal) -> Principal {
    // * config for provisioning canister
    let arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![
                // * this user_index canister
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
        profile_owner: Some(profile_owner),
        known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell.borrow().known_principal_ids.clone()
        })),
        upgrade_version_number: Some(0),
        url_to_send_canister_metrics_to: Some(configuration.url_to_send_canister_metrics_to),
    };

    // * encode argument for user canister init lifecycle method
    let arg = candid::encode_one(individual_user_tempalate_init_args)
        .expect("Failed to serialize the install argument.");

    // * install wasm to provisioned canister
    main::install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: INDIVIDUAL_USER_TEMPLATE_CANISTER_WASM.into(),
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
    unsafe_drop_stable_memory: bool
) -> Result<(), (RejectionCode, String)> {
    stop_canister(CanisterIdRecord {canister_id: canister_id.clone()}).await?;
    loop {
        let (canister_status, ) = canister_status(CanisterIdRecord {canister_id: canister_id.clone()}).await.unwrap();
        match canister_status.status {
            CanisterStatusType::Stopped => break,
            _ => ic_cdk::println!("Canister {:?} is stopping", &canister_id)
        }
    }        
        

    let serialized_arg =
        candid::encode_args((arg,)).expect("Failed to serialize the install argument.");

        let upgrade_args = CustomInstallCodeArgument {
            mode: install_mode,
            canister_id: canister_id.clone(),
            wasm_module: INDIVIDUAL_USER_TEMPLATE_CANISTER_WASM.into(),
            sender_canister_version: Some(canister_version()),
            arg: serialized_arg,
            unsafe_drop_stable_memory: Some(unsafe_drop_stable_memory)
        };

    api::call::call(Principal::management_canister(), "install_code", (upgrade_args, )).await?;

    start_canister(CanisterIdRecord {canister_id}).await
}