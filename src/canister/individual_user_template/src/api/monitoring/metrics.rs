use ic_cdk::api;
use shared_utils::common::utils::{get_heap_memory_size, get_stable_memory_size};

use crate::CANISTER_DATA;

pub fn metrics() -> String {
    // Prometheus expects timestamps in ms. Time.now() returns ns.
    let timestamp = api::time() / 1000000;
    let version =
        CANISTER_DATA.with(|canister_data| canister_data.borrow().version_details.version_number);
    let principal =
        match CANISTER_DATA.with(|canister_data| canister_data.borrow().profile.principal_id) {
            Some(principal) => principal.to_string(),
            None => "None".to_string(),
        };
    let canister_type = match principal.as_str() {
        "None" => "anonymous".to_string(),
        _ => "authenticated".to_string(),
    };

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
        format!("version {} {}", version, timestamp).as_str(),
        format!("profile{{principal=\"{}\"}} 1 {}", principal, timestamp).as_str(),
    ]
    .join("\n")
}
