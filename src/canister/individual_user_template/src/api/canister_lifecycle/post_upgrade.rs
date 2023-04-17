use std::time::Duration;

use shared_utils::common::utils::{stable_memory_serializer_deserializer, system_time};

use crate::{
    api::{
        hot_or_not_bet::{
            reenqueue_timers_for_pending_bet_outcomes::reenqueue_timers_for_pending_bet_outcomes,
            tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot,
        },
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
    one_time_calculate_pending_slot_scores();
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

// TODO: remove this on next update
const DELAY_FOR_TABULATING_PENDING_SLOTS: Duration = Duration::from_secs(2);
fn one_time_calculate_pending_slot_scores() {
    ic_cdk_timers::set_timer(DELAY_FOR_TABULATING_PENDING_SLOTS, || {
        calculate_pending_slot_outcomes();
    });
}

fn calculate_pending_slot_outcomes() {
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();

        let posts_not_created_in_the_last_48_hours_with_betting_enabled: Vec<u64> = canister_data
            .all_created_posts
            .iter()
            .take_while(|(_post_id, post)| {
                let post_not_created_in_the_last_48_hours = current_time
                    .duration_since(post.created_at)
                    .unwrap()
                    .as_secs()
                    > 48 * 60 * 60;
                let is_a_hot_or_not_post = post.hot_or_not_details.is_some();

                post_not_created_in_the_last_48_hours && is_a_hot_or_not_post
            })
            .map(|(post_id, _post)| *post_id)
            .collect();

        for post_id in posts_not_created_in_the_last_48_hours_with_betting_enabled {
            let post = canister_data.all_created_posts.get(&post_id).unwrap();

            let hot_or_not_details = post.hot_or_not_details.as_ref().unwrap();

            hot_or_not_details
                .slot_history
                .iter()
                .for_each(|(slot_id, _slot_detail)| {
                    tabulate_hot_or_not_outcome_for_post_slot(
                        &mut canister_data_ref_cell.borrow_mut(),
                        post_id,
                        *slot_id,
                    );
                })
        }
    });
}
