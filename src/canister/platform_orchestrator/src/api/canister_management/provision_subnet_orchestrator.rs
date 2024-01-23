use std::{iter::Cycle, ops::Sub, str::FromStr, vec};

use candid::{Principal, CandidType, utils::ArgumentEncoder};
use ic_cdk::{api::{management_canister::{provisional::{CanisterIdRecord, CanisterSettings}, main::{self, create_canister, delete_canister, deposit_cycles, stop_canister, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument}}, call, self}, id};
use serde::{Deserialize, Serialize};
use shared_utils::{constant::{CYCLES_THRESHOLD_TO_INITIATE_RECHARGE, INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT, NNS_CYCLE_MINTING_CANISTER}, canister_specific::user_index::types::args::UserIndexInitArgs};

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

#[candid::candid_method(update)]
#[ic_cdk::update]
pub async fn provision_subnet_orchestrator_canister(subnet: Principal, user_index_wasm: Vec<u8>) -> Principal {

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
       INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT as u64
    )
    .await
    .unwrap();

    let subnet_orchestrator_canister_id = res.unwrap();
    
    // Create PostCache Canister


    //TODO: Where should we send the known principal ids

    let user_index_init_arg = UserIndexInitArgs {
        known_principal_ids: None,
        access_control_map: None,
        version: CANISTER_DATA.with_borrow(|canister_data| canister_data.version_detail.version.clone())
    };

    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: subnet_orchestrator_canister_id,
        wasm_module: user_index_wasm.into(),
        arg: candid::encode_one(user_index_init_arg).unwrap()
    };

    main::install_code(install_code_arg).await.unwrap();


    subnet_orchestrator_canister_id

}



#[candid::candid_method(update)]
#[ic_cdk::update]
pub async fn delete_all_caniter(canister_id: Principal) {



    stop_canister(CanisterIdRecord {
        canister_id
    }).await.unwrap();
    delete_canister(CanisterIdRecord {
        canister_id
    }).await.unwrap();
}

