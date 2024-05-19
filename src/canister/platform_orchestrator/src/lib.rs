use candid::Principal;
use std::cell::RefCell;

use crate::api::generic_proposal::{
    PlatformOrchestratorGenericArgumentType, PlatformOrchestratorGenericResultType,
};
use crate::data_model::CanisterUpgradeStatus;
use data_model::CanisterData;
use ic_cdk_macros::export_candid;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::{
        PlatformOrchestratorInitArgs, UpgradeCanisterArg,
    },
    common::types::http::{HttpRequest, HttpResponse},
    common::types::known_principal::KnownPrincipalType,
    common::types::wasm::WasmType,
};

mod api;
mod data_model;
mod guard;
mod utils;

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
