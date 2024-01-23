use std::cell::RefCell; 

use candid::{export_service, Principal};

use data_model::CanisterData;
use shared_utils::canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs;

mod data_model;
#[cfg(test)]
mod test;
mod api;





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