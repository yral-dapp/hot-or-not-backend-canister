use std::borrow::Borrow;

use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use shared_utils::canister_specific::individual_user_template::types::kv_storage::{
    NamespaceErrors, NamespaceForFrontend,
};

use crate::{
    data_model::kv_storage::{AppStorage, Namespace},
    CANISTER_DATA,
};

#[update]
fn create_a_namespace(title: String) -> Result<NamespaceForFrontend, NamespaceErrors> {
    AppStorage::create_a_namespace(caller(), title)
}

#[query]
fn list_namespaces(start_index: usize, limit: usize) -> Vec<NamespaceForFrontend> {
    AppStorage::list_namespaces(start_index, limit)
}
