use candid::candid_method;
use ic_cdk::{api::is_controller, caller};
use shared_utils::{common::{types::wasm::{CanisterWasm, WasmType}, utils::task::run_task_concurrently}, constant::INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE};

use crate::{util::canister_management::create_users_canister, CANISTER_DATA};


#[ic_cdk::update]
#[candid_method(update)]
pub fn create_pool_of_individual_user_available_canisters(version: String, individual_user_wasm: Vec<u8>) -> Result<String, String> {
    
    if !is_controller(&caller()) {
        return Err("Unauthorized".into())
    }

    //store wasm internally
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.wasms.insert(WasmType::IndividualUserWasm, CanisterWasm {
            version: version.clone(),
            wasm_blob: individual_user_wasm.clone()
        })
    });
     
   ic_cdk::spawn(impl_create_pool_of_individual_user_available_canisters(INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE, version, individual_user_wasm)); 
   Ok("Success".into())
}


pub async fn impl_create_pool_of_individual_user_available_canisters(limit: u64, version: String, individual_user_wasm: Vec<u8>) {
    let indiviual_canister_futures = (0..limit).map(|_| {
        create_users_canister(None, version.clone(), individual_user_wasm.clone())
    });

    let result_callback = |res| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.available_canisters.insert(res)
        });
    };

    run_task_concurrently(indiviual_canister_futures, 10, result_callback, || false).await;
}