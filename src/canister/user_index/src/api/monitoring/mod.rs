use std::borrow::Cow;

use candid::{candid_method, types::principal, CandidType};
use ic_cdk::api;
use serde::Deserialize;
use shared_utils::common::{
    types::http::{HeaderField, HttpRequest, HttpResponse},
    utils::{get_heap_memory_size, get_stable_memory_size},
};

pub fn metrics() -> String {
    // Prometheus expects timestamps in ms. Time.now() returns ns.
    let timestamp = api::time() / 1000000;
    let canister_type = "user_index";

    vec![
        format!(
            "cycle_balance{{type=\"{}\"}} {} {}",
            canister_type,
            api::canister_balance128(),
            timestamp
        )
        .as_str(),
        format!(
            "heap_size{{type=\"{}\"}} {} {}",
            canister_type,
            get_heap_memory_size(),
            timestamp
        )
        .as_str(),
        format!(
            "stable_size{{type=\"{}\"}} {} {}",
            canister_type,
            get_stable_memory_size(),
            timestamp
        )
        .as_str(),
    ]
    .connect("\n")
}
