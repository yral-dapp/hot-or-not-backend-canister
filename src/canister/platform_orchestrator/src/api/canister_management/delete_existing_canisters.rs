use candid::Principal;
use ic_cdk::{api::{is_controller, management_canister::{main::{delete_canister, stop_canister}, provisional::CanisterIdRecord}}, caller};
use ic_cdk_macros::update;

use crate::CANISTER_DATA;



#[update]
async fn delete_existing_subnet_canisters() -> Result<String, String> {
    if !is_controller(&caller()) {
        return Err("Unauthorized".into());
    }


    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let _ = canister_data.all_subnet_orchestrator_canisters_list.iter().map (|canister_id| async {
            stop_and_delete_canister(*canister_id).await;  
        });

        let _ = canister_data.all_post_cache_orchestrator_list.iter().map (|canister_id| async {
            stop_and_delete_canister(*canister_id).await;  
        });
    });

    

    Ok("Success".into())
}


// async fn delete_all_canister(iter: Iterator<Principal>) {
//     while iter.
// }

async fn stop_and_delete_canister(canister_id: Principal) {
    stop_canister(CanisterIdRecord {
        canister_id
    })
    .await
    .unwrap();

    delete_canister(CanisterIdRecord {
        canister_id
    })
    .await
    .unwrap();
}