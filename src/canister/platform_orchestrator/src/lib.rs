use std::cell::RefCell; 

use candid::export_service;

use data_model::CanisterData;
use candid::Principal;
use shared_utils::{canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs, common::types::wasm::WasmType};
use crate::data_model::{UpgradeCanisterArg, CanisterUpgradeStatus};

mod data_model;
#[cfg(test)]
mod test;
mod api;


//TODO: add method to deposit cycle



thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

/*
write a function to provison user_index canister on a subnet.

 */