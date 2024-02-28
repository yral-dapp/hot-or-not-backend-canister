use ic_cdk::{api::is_controller, caller};
use shared_utils::{common::{types::wasm::{CanisterWasm, WasmType}, utils::task::run_task_concurrently}, constant::{BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE, INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE}};
use ic_cdk_macros::update;

use crate::{util::canister_management::{create_empty_user_canister, create_users_canister}, CANISTER_DATA};


#[update]
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
     
   ic_cdk::spawn(impl_create_pool_of_individual_user_available_canisters(version, individual_user_wasm)); 
   Ok("Success".into())
}


pub async fn impl_create_pool_of_individual_user_available_canisters(version: String, individual_user_wasm: Vec<u8>) {
    
    //Canisters with installed wasm
    for _ in 0..INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE {
        let canister_id = create_users_canister(None, version.clone(), individual_user_wasm.clone()).await;
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.available_canisters.insert(canister_id);
        })
    }

    //Empty backup canisters
    for _ in 0 .. BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE {
        let canister_id = create_empty_user_canister().await;
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.backup_canister_pool.insert(canister_id);
        })
    }
}