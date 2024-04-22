use std::time::SystemTime;

use candid::Principal;
use futures::StreamExt;
use ic_cdk::api::management_canister::main::{self, CanisterIdRecord};
use shared_utils::{
    common::types::wasm::{CanisterWasm, WasmType},
    constant::CANISTER_RECYCLING_THRESHOLD,
};

use crate::{
    data_model::recycle_canister::RecycleStatus, util::canister_management::install_canister_wasm,
    CANISTER_DATA,
};

pub async fn recycle_canisters_job() {
    // 1. iterate all canisters in user_principal_id_to_canister_id_map
    // 2. call get_last_canister_functionality_access_time for each canister
    // 3. If the canister has not been accessed for more than 7 days, add it to the list of canisters to be recycled
    // 4. call reset_user_individual_canister with the list of canisters to be recycled

    let canisters_list = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .iter()
            .map(|item| *item.1)
            .collect::<Vec<Principal>>()
    });

    let individual_user_template_canister_wasm = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .wasms
            .get(&WasmType::IndividualUserWasm)
            .unwrap()
            .clone()
    });

    let futures = canisters_list.iter().map(|canister_id| async {
        recycle_canister(*canister_id, individual_user_template_canister_wasm.clone()).await
    });

    let stream = futures::stream::iter(futures).boxed().buffer_unordered(10);

    let results = stream
        .collect::<Vec<Result<Option<Principal>, (Principal, String)>>>()
        .await;

    // update recycle_status

    let success_canisters = results
        .iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_some())
        .map(|r| r.as_ref().unwrap().unwrap())
        .collect::<Vec<Principal>>();

    let num_success = success_canisters.len();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .available_canisters
            .extend(success_canisters.clone());

        // remove the canisters that are recycled from user_principal_id_to_canister_id_map and unique_user_name_to_user_principal_id_map
        // canister_id is the value in the map
        for canister_id in success_canisters {
            canister_data
                .user_principal_id_to_canister_id_map
                .retain(|_, v| v != &canister_id);
            canister_data
                .unique_user_name_to_user_principal_id_map
                .retain(|_, v| v != &canister_id);
        }
    });

    let failed_list = results
        .iter()
        .filter(|r| r.is_err())
        .map(|r| r.as_ref().unwrap_err().clone())
        .collect::<Vec<(Principal, String)>>();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.recycle_status = RecycleStatus {
            last_recycled_at: Some(SystemTime::now()),
            num_last_recycled_canisters: num_success as u64,
            failed_recycling: failed_list,
        };
    });
}

pub async fn recycle_canister(
    canister_id: Principal,
    individual_user_template_canister_wasm: CanisterWasm,
) -> Result<Option<Principal>, (Principal, String)> {
    let (last_access_time,): (Option<SystemTime>,) = match ic_cdk::call(
        canister_id,
        "get_last_canister_functionality_access_time",
        (),
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err((canister_id, e.1));
        }
    };

    if let Some(last_access_time) = last_access_time {
        let now = SystemTime::now();
        let duration = now.duration_since(last_access_time).unwrap();
        if duration.as_secs() > CANISTER_RECYCLING_THRESHOLD {
            // reinstall wasm and add to available canisters
            // uninstall code
            match main::uninstall_code(CanisterIdRecord { canister_id }).await {
                Ok(_) => {}
                Err(e) => {
                    return Err((canister_id, e.1));
                }
            }

            install_canister_wasm(
                canister_id,
                None,
                individual_user_template_canister_wasm.version.clone(),
                individual_user_template_canister_wasm.wasm_blob.clone(),
            )
            .await;
        } else {
            return Ok(None);
        }
    }

    Ok(Some(canister_id))
}
