use std::time::Duration;

use candid::Principal;
use ic_cdk::api::management_canister::{
    http_request::{self, CanisterHttpRequestArgument, HttpMethod},
    main,
    provisional::CanisterIdRecord,
};
use rmp_serde::encode;
use serde::Serialize;

// Send metrics every 6 hours
const PING_INTERVAL_FOR_CALLING_METRICS_REST_API: Duration = Duration::from_secs(60 * 60 * 6);

pub fn enqueue_timer_for_calling_metrics_rest_api(url_to_ping: String) {
    ic_cdk_timers::set_timer_interval(PING_INTERVAL_FOR_CALLING_METRICS_REST_API, move || {
        ic_cdk::spawn(ping_metrics_rest_api(url_to_ping.clone()))
    });
}

async fn ping_metrics_rest_api(url_to_ping: String) {
    let status = get_my_canister_cycle_balance_and_memory_usage().await;

    let request_arg = CanisterHttpRequestArgument {
        url: url_to_ping,
        max_response_bytes: Some(0),
        method: HttpMethod::POST,
        body: Some(encode::to_vec(&status).expect("Failed to serialize status")),
        ..Default::default()
    };

    http_request::http_request(request_arg)
        .await
        .expect("Failed to ping");
}

#[derive(Serialize)]
pub struct CanisterStatus {
    pub canister_id: Principal,
    cycle_balance: i64,
    idle_cycles_burned_per_day: i64,
    memory_size: i64,
}

#[derive(Serialize)]
pub struct CanisterStatusError {
    pub canister_id: Principal,
    pub error_message: String,
}

pub async fn get_my_canister_cycle_balance_and_memory_usage(
) -> Result<CanisterStatus, CanisterStatusError> {
    let canister_id = ic_cdk::id();

    let canister_status_result = main::canister_status(CanisterIdRecord { canister_id })
        .await
        .map_err(|err| CanisterStatusError {
            canister_id: ic_cdk::id(),
            error_message: err.1,
        })?
        .0;

    Ok(CanisterStatus {
        canister_id,
        cycle_balance: canister_status_result.cycles.0.try_into().unwrap(),
        memory_size: canister_status_result.memory_size.0.try_into().unwrap(),
        idle_cycles_burned_per_day: canister_status_result
            .idle_cycles_burned_per_day
            .0
            .try_into()
            .unwrap(),
    })
}
