use std::{str::FromStr, vec};
use candid::{Principal, CandidType};
use ic_cdk::{api::{self, call, is_controller, management_canister::{main::{self,  CanisterInstallMode, InstallCodeArgument}, provisional::CanisterSettings}}, call, caller, id};
use ic_cdk_macros::update;
use serde::{Deserialize, Serialize};
use shared_utils::{canister_specific::{post_cache::types::arg::PostCacheInitArgs, user_index::types::args::UserIndexInitArgs}, common::types::{known_principal::{KnownPrincipalMap, KnownPrincipalType}, wasm::WasmType}, constant::{GLOBAL_SUPER_ADMIN_USER_ID, INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT, NNS_CYCLE_MINTING_CANISTER, POST_CACHE_CANISTER_CYCLES_RECHARGE_AMOUMT, SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES}};

use crate::CANISTER_DATA;



#[derive(CandidType, Serialize)]
enum SubnetType {
    Filter(Option<String>),
    Subnet(Subnet)

}

#[derive(CandidType, Serialize)]
struct Subnet {
    subnet: Principal
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq, Eq)]
pub enum CmcCreateCanisterError {
    Refunded {
        refund_amount: u128,
        create_error: String,
    },
    RefundFailed {
        create_error: String,
        refund_error: String,
    },
}

#[derive(CandidType, Serialize)]
struct CreateCanisterCmcArgument {
    subnet_selection: Option<SubnetType>,
    canister_settings: Option<CanisterSettings>,
    subnet_type: Option<String>
}

#[update]
pub async fn provision_subnet_orchestrator_canister(subnet: Principal) -> Result<Principal, String> {
    
    if !is_controller(&caller()) {
        return Err("Unauthorized".into());
    }

    let create_canister_arg = CreateCanisterCmcArgument {
        subnet_selection: Some(SubnetType::Subnet(Subnet {
            subnet
        })),
        canister_settings: Some(CanisterSettings {
            controllers: Some(vec![ api::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
        subnet_type: None
    };
    let (res, ): (Result<Principal, CmcCreateCanisterError>, ) = call::call_with_payment(
        Principal::from_str(NNS_CYCLE_MINTING_CANISTER).unwrap(), 
        "create_canister",
       (create_canister_arg,),
       SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES as u64
    )
    .await
    .unwrap();

    let subnet_orchestrator_canister_id = res.unwrap();
    
    let create_canister_arg = CreateCanisterCmcArgument {
        subnet_selection: Some(SubnetType::Subnet(Subnet {
            subnet
        })),
        canister_settings: Some(CanisterSettings {
            controllers: Some(vec![ api::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
        subnet_type: None
    };
    let (res, ): (Result<Principal, CmcCreateCanisterError>, ) = call::call_with_payment(
        Principal::from_str(NNS_CYCLE_MINTING_CANISTER).unwrap(), 
        "create_canister",
       (create_canister_arg,),
        POST_CACHE_CANISTER_CYCLES_RECHARGE_AMOUMT as u64
    )
    .await
    .unwrap();

    let post_cache_canister_id = res.unwrap();




    let mut known_principal_map = KnownPrincipalMap::default();
    known_principal_map.insert(KnownPrincipalType::CanisterIdPlatformOrchestrator, id());
    known_principal_map.insert(KnownPrincipalType::CanisterIdUserIndex, subnet_orchestrator_canister_id);
    known_principal_map.insert(KnownPrincipalType::CanisterIdPostCache, post_cache_canister_id);
    known_principal_map.insert(KnownPrincipalType::UserIdGlobalSuperAdmin, Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap());

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.all_post_cache_orchestrator_list.insert(post_cache_canister_id);
        canister_data.all_subnet_orchestrator_canisters_list.insert(subnet_orchestrator_canister_id);
        canister_data.subet_orchestrator_with_capacity_left.insert(subnet_orchestrator_canister_id);
    });

    let user_index_init_arg = UserIndexInitArgs {
        known_principal_ids: Some(known_principal_map.clone()),
        access_control_map: None,
        version: CANISTER_DATA.with_borrow(|canister_data| canister_data.version_detail.version.clone())
    };

    let subnet_orchestrator_install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: subnet_orchestrator_canister_id,
        wasm_module: CANISTER_DATA.with_borrow(|canister_data| canister_data.wasms.get(&WasmType::SubnetOrchestratorWasm).unwrap().wasm_blob),
        arg: candid::encode_one(user_index_init_arg).unwrap()
    };

    main::install_code(subnet_orchestrator_install_code_arg).await.unwrap();

    let post_cache_canister_wasm = CANISTER_DATA.with_borrow(|canister_data| canister_data.wasms.get(&WasmType::PostCacheWasm).unwrap());

    let post_cache_init_arg = PostCacheInitArgs {
        version: post_cache_canister_wasm.version,
        upgrade_version_number: None,
        known_principal_ids: Some(known_principal_map)
   };

   let post_cache_install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: post_cache_canister_id,
        wasm_module: post_cache_canister_wasm.wasm_blob.clone(),
        arg: candid::encode_one(post_cache_init_arg).unwrap()
    };

    main::install_code(post_cache_install_code_arg).await.unwrap();

    //provision pool of canisters
    let individual_user_template_wasm = CANISTER_DATA.with_borrow(|canister_data| canister_data.wasms.get(&WasmType::IndividualUserWasm).clone()).unwrap();

    let (create_pool_of_individual_canister_res, ): (Result<String, String>, ) = call(subnet_orchestrator_canister_id, "create_pool_of_individual_user_available_canisters", (individual_user_template_wasm.version, individual_user_template_wasm.wasm_blob)).await.unwrap();

    create_pool_of_individual_canister_res.unwrap();
    
    Ok(subnet_orchestrator_canister_id)

}
