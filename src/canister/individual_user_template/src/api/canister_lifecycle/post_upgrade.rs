use std::time::Duration;

use shared_utils::common::utils::stable_memory_serializer_deserializer;

use crate::{
    api::{
        hot_or_not_bet::reenqueue_timers_for_pending_bet_outcomes::reenqueue_timers_for_pending_bet_outcomes,
        well_known_principal::update_locally_stored_well_known_principals,
    },
    data_model::CanisterData,
    CANISTER_DATA,
};

use super::pre_upgrade::BUFFER_SIZE_BYTES;

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    refetch_well_known_principals();
    reenqueue_timers_for_pending_bet_outcomes();
}

fn restore_data_from_stable_memory() {
    match stable_memory_serializer_deserializer::deserialize_from_stable_memory::<CanisterData>(
        BUFFER_SIZE_BYTES,
    ) {
        Ok(canister_data) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    }
}

const DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS: Duration = Duration::from_secs(1);
fn refetch_well_known_principals() {
    ic_cdk_timers::set_timer(DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS, || {
        ic_cdk::spawn(update_locally_stored_well_known_principals::update_locally_stored_well_known_principals())
    });
}
