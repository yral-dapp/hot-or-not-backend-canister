use std::cell::RefCell;

use candid::{export_service, Principal};
use data::CanisterData;
use shared_utils::{
    access_control::UserAccessRole,
    canister_specific::configuration::types::args::ConfigurationInitArgs,
    common::types::known_principal::KnownPrincipalType,
};

use crate::api::well_known_principal::update_list_of_well_known_principals::ErrorUpdateListOfWellKnownPrincipals;

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
