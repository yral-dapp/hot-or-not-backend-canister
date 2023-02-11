use ic_cdk::api;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_current_system_time_from_ic() -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::new(
            api::time() / 1_000_000_000,
            (api::time() % 1_000_000_000) as u32,
        ))
        .expect("Getting timestamp from ic_cdk failed")
}
