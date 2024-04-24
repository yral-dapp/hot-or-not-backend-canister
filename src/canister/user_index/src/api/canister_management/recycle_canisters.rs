use std::time::SystemTime;

use candid::Principal;
use futures::StreamExt;
use ic_cdk::{
    api::management_canister::main::{self, CanisterIdRecord},
    query, update,
};
use serde::Deserialize;
use shared_utils::{
    canister_specific::user_index::types::RecycleStatus,
    common::{
        types::wasm::{CanisterWasm, WasmType},
        utils::{permissions::is_caller_controller, system_time::get_current_system_time},
    },
};

use crate::{util::canister_management::reinstall_canister_wasm, CANISTER_DATA};

#[query]
pub fn get_recycle_status() -> RecycleStatus {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.recycle_status.clone())
}

#[derive(Deserialize, Debug)]
struct RecycleCanistersRequest {
    canister_ids: Vec<Principal>,
}

pub fn handle_recycle_canisters(body: Vec<u8>) -> String {
    let req: RecycleCanistersRequest = serde_json::from_slice(&body).unwrap();
    let canister_ids = req.canister_ids;

    recycle_canisters(canister_ids);

    "Recycling canisters started".to_string()
}

pub fn recycle_canisters(canister_ids: Vec<Principal>) {
    ic_cdk::spawn(async move {
        let start = get_current_system_time();

        let individual_user_template_canister_wasm = CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .wasms
                .get(&WasmType::IndividualUserWasm)
                .unwrap()
                .clone()
        });

        let futures = canister_ids.iter().map(|canister_id| {
            recycle_canister(*canister_id, individual_user_template_canister_wasm.clone())
        });

        let stream = futures::stream::iter(futures).boxed().buffer_unordered(20);

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
            for canister_id in success_canisters.clone() {
                canister_data
                    .user_principal_id_to_canister_id_map
                    .retain(|_, v| *v != canister_id);
                canister_data
                    .unique_user_name_to_user_principal_id_map
                    .retain(|_, v| *v != canister_id);
            }
        });

        let failed_list = results
            .iter()
            .filter(|r| r.is_err())
            .map(|r| r.as_ref().unwrap_err().clone())
            .collect::<Vec<(Principal, String)>>();

        let end = get_current_system_time();

        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data.recycle_status = RecycleStatus {
                success_canisters: success_canisters.iter().map(|c| c.to_string()).collect(),
                last_recycled_at: Some(end),
                last_recycled_duration: Some(end.duration_since(start).unwrap().as_secs()),
                num_last_recycled_canisters: num_success as u64,
                failed_recycling: failed_list,
            };
        });
    });
}

pub async fn recycle_canister(
    canister_id: Principal,
    individual_user_template_canister_wasm: CanisterWasm,
) -> Result<Option<Principal>, (Principal, String)> {
    // reinstall wasm
    match reinstall_canister_wasm(
        canister_id,
        None,
        individual_user_template_canister_wasm.version.clone(),
        individual_user_template_canister_wasm.wasm_blob.clone(),
    )
    .await
    {
        Ok(_) => Ok(Some(canister_id)),
        Err(e) => Err((canister_id, e)),
    }
}
