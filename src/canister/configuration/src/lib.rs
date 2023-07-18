use std::cell::RefCell;

use candid::{export_service, Principal};
use data::CanisterData;
use shared_utils::{
    canister_specific::configuration::types::args::ConfigurationInitArgs,
    common::types::known_principal::KnownPrincipalType,
};

mod api;
mod data;
#[cfg(test)]
mod test;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
