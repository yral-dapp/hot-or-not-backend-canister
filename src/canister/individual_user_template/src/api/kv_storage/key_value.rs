use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use shared_utils::canister_specific::individual_user_template::types::kv_storage::NamespaceErrors;
use std::collections::BTreeMap;

use crate::data_model::kv_storage::AppStorage;

#[update]
fn delete_multiple_key_value_pairs(
    namespace_id: u64,
    keys: Vec<String>,
) -> Result<(), NamespaceErrors> {
    let namespace = AppStorage::get_a_namespace(caller(), namespace_id)?;
    namespace.delete_multiple_keys(keys);
    Ok(())
}

#[update]
fn write_multiple_key_value_pairs(
    namespace_id: u64,
    pairs: BTreeMap<String, String>,
) -> Result<(), NamespaceErrors> {
    let namespace = AppStorage::get_a_namespace(caller(), namespace_id)?;
    namespace.write_multiple_key_value_pairs(pairs)
}

#[update]
fn write_key_value_pair(
    namespace_id: u64,
    key: String,
    value: String,
) -> Result<Option<String>, NamespaceErrors> {
    let namespace = AppStorage::get_a_namespace(caller(), namespace_id)?;
    let prev_value = namespace.write_key_value_pair(key, value)?;
    Ok(prev_value)
}

#[query]
fn list_namespace_keys(namespace_id: u64) -> Result<Vec<String>, NamespaceErrors> {
    let namespace = AppStorage::get_a_namespace(caller(), namespace_id)?;
    Ok(namespace.list_keys())
}

#[update]
fn delete_key_value_pair(
    namespace_id: u64,
    key: String,
) -> Result<Option<String>, NamespaceErrors> {
    let namespace = AppStorage::get_a_namespace(caller(), namespace_id)?;
    Ok(namespace.delete_key_value_pair(key))
}

#[query]
fn read_key_value_pair(namespace_id: u64, key: String) -> Result<Option<String>, NamespaceErrors> {
    let namespace = AppStorage::get_a_namespace(caller(), namespace_id)?;
    Ok(namespace.read_key_value_pair(key))
}
