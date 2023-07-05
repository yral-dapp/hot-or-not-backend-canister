use std::cell::RefCell;

use candid::{export_service, Principal};
use data_model::{canister_upgrade::upgrade_status::UpgradeStatus, CanisterData};
use ic_cdk::api::management_canister::main::CanisterInstallMode;
use shared_utils::{
    canister_specific::user_index::types::args::UserIndexInitArgs,
    common::types::known_principal::KnownPrincipalType,
    types::canister_specific::user_index::error_types::SetUniqueUsernameError,
};

mod api;
mod data_model;
#[cfg(test)]
mod test;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
