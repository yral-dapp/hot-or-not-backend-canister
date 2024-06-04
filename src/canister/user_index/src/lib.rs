use std::cell::RefCell;

use candid::Principal;
use data_model::{canister_upgrade::UpgradeStatus, CanisterData};
use ic_cdk::api::{
    call::CallResult,
    management_canister::main::{CanisterInstallMode, CanisterStatusResponse},
};
use ic_cdk_macros::export_candid;
use shared_utils::{
    canister_specific::user_index::types::{
        args::UserIndexInitArgs, BroadcastCallStatus, RecycleStatus,
    },
    common::types::http::{HttpRequest, HttpResponse},
    common::types::known_principal::KnownPrincipalType,
    types::canister_specific::user_index::error_types::SetUniqueUsernameError,
};

mod api;
mod data_model;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
