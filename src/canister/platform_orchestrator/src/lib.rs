use std::cell::RefCell; 
use candid::Principal;

use data_model::CanisterData;
use ic_cdk_macros::export_candid;
use shared_utils::{canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs, common::types::wasm::WasmType};
use crate::data_model::{UpgradeCanisterArg, CanisterUpgradeStatus};

mod data_model;
mod api;


//TODO: add method to deposit cycle

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();