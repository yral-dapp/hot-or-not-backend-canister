use futures::StreamExt;
use ic_cdk::{api::is_controller, caller};
use shared_utils::{common::types::wasm::{CanisterWasm, WasmType}, constant::{get_backup_individual_user_canister_batch_size, get_backup_individual_user_canister_threshold, get_individual_user_canister_subnet_batch_size}};
use ic_cdk_macros::update;

use crate::{util::canister_management::{create_empty_user_canister, install_canister_wasm, recharge_canister_for_installing_wasm}, CANISTER_DATA};


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

    let backup_individual_user_canister_batch_size = get_backup_individual_user_canister_batch_size();
    let individual_user_canister_subnet_batch_size = get_individual_user_canister_subnet_batch_size();
    let total_cnt = backup_individual_user_canister_batch_size + individual_user_canister_subnet_batch_size;

    //empty canister for backup
    let create_empty_canister_futures = (0..total_cnt)
        .map(|_| create_empty_user_canister());
    let mut cans_stream = futures::stream::iter(create_empty_canister_futures).buffer_unordered(10);

    let to_install: Vec<_> = cans_stream.by_ref().take(individual_user_canister_subnet_batch_size as usize).collect().await;
    CANISTER_DATA.with_borrow_mut(|cdata| {
        cdata.children_merkle.insert_children(to_install.clone());
    });

    //canisters with installed wasm for available pool
    let install_wasm_futs = to_install.into_iter().map(move |canister_id| {
        let version = version.clone();
        let individual_user_wasm = individual_user_wasm.clone();
        async move {
            recharge_canister_for_installing_wasm(canister_id).await.map_err(|e| (canister_id, e))?;
            install_canister_wasm(
                canister_id,
                None,
                version,
                individual_user_wasm,
            ).await
        }
    });
    let mut install_wasm_stream = futures::stream::iter(install_wasm_futs).buffer_unordered(10);
    while let Some(res) = install_wasm_stream.next().await {
        match res {
            Ok(canister_id) => {
                CANISTER_DATA.with_borrow_mut(|cdata| {
                    cdata.available_canisters.insert(canister_id);
                })
            }
            Err((canister_id, e)) => {
                ic_cdk::println!("Failed to install wasm on canister: {:?}, error: {:?}", canister_id, e);
                CANISTER_DATA.with_borrow_mut(|cdata| {
                    cdata.insert_backup_canister(canister_id);
                })
            }
        }
    }

    while let Some(canister_id) = cans_stream.next().await {
        CANISTER_DATA.with_borrow_mut(|cdata| {
            cdata.insert_backup_canister(canister_id);
        })
    }
}