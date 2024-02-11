use ciborium::de;
use ic_stable_structures::Memory;
use std::{borrow::BorrowMut, time::Duration};

use crate::data_model::memory;

use shared_utils::canister_specific::individual_user_template::types::{
    arg::IndividualUserTemplateInitArgs,
    hot_or_not::{
        BetDirection, BetMakerPrincipal, GlobalBetId, GlobalRoomId, RoomDetailsV1, StablePrincipal,
    },
};

use crate::{
    api::{
        hot_or_not_bet::reenqueue_timers_for_pending_bet_outcomes::reenqueue_timers_for_pending_bet_outcomes,
        well_known_principal::update_locally_stored_well_known_principals,
    },
    CANISTER_DATA,
};

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    save_upgrade_args_to_memory();
    refetch_well_known_principals();
    reenqueue_timers_for_pending_bet_outcomes();
    migrate_room_bets_details_to_stable_memory();
}

fn restore_data_from_stable_memory() {
    let heap_data = memory::get_upgrades_memory();
    let mut heap_data_len_bytes = [0; 4];
    heap_data.read(0, &mut heap_data_len_bytes);
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    let mut canister_data_bytes = vec![0; heap_data_len];
    heap_data.read(4, &mut canister_data_bytes);

    let canister_data =
        de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");
    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data;
    });
}

fn save_upgrade_args_to_memory() {
    let upgrade_args = ic_cdk::api::call::arg_data::<(IndividualUserTemplateInitArgs,)>().0;

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        if let Some(known_principal_ids) = upgrade_args.known_principal_ids {
            canister_data_ref_cell.known_principal_ids = known_principal_ids;
        }

        if let Some(profile_owner) = upgrade_args.profile_owner {
            canister_data_ref_cell.profile.principal_id = Some(profile_owner);
        }

        if let Some(upgrade_version_number) = upgrade_args.upgrade_version_number {
            canister_data_ref_cell.version_details.version_number = upgrade_version_number;
        }

        canister_data_ref_cell.borrow_mut().version_details.version = upgrade_args.version;

        if let Some(url_to_send_canister_metrics_to) = upgrade_args.url_to_send_canister_metrics_to
        {
            canister_data_ref_cell
                .configuration
                .url_to_send_canister_metrics_to = Some(url_to_send_canister_metrics_to);
        }
    });
}

const DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS: Duration = Duration::from_secs(1);
fn refetch_well_known_principals() {
    ic_cdk_timers::set_timer(DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS, || {
        ic_cdk::spawn(update_locally_stored_well_known_principals::update_locally_stored_well_known_principals())
    });
}

const DELAY_FOR_MIGRATING_DATA: Duration = Duration::from_secs(1);
fn migrate_room_bets_details_to_stable_memory() {
    ic_cdk_timers::set_timer(DELAY_FOR_MIGRATING_DATA, || {
        migrate_room_bets_details_to_stable_memory_impl()
    });
}

fn migrate_room_bets_details_to_stable_memory_impl() {
    let posts = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_ref_cell = canister_data_ref_cell.borrow_mut();

        canister_data_ref_cell.all_created_posts.clone()
    });

    let room_details = posts
        .iter()
        .filter_map(|(post_id, post)| {
            if let Some(hot_or_not_details) = &post.hot_or_not_details {
                Some((post_id, hot_or_not_details))
            } else {
                None
            }
        })
        .map(|(post_id, hot_or_not_details)| {
            hot_or_not_details
                .slot_history
                .iter()
                .map(move |(slot_id, slot_details)| (post_id, slot_id, slot_details))
                .map(|(post_id, slot_id, slot_details)| {
                    slot_details
                        .room_details
                        .iter()
                        .map(move |(room_id, room_details)| {
                            (post_id, slot_id, room_id, room_details)
                        })
                })
        })
        .flatten()
        .flatten();

    room_details.for_each(|(post_id, slot_id, room_id, room_details)| {
        let global_room_id = GlobalRoomId(post_id.clone(), slot_id.clone(), room_id.clone());

        room_details.bets_made.iter().for_each(|(bet_maker, bet)| {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
                let global_bet_id =
                    GlobalBetId(global_room_id.clone(), StablePrincipal(bet_maker.clone()));
                canister_data_ref_cell
                    .bet_details_map
                    .insert(global_bet_id, bet.clone());
            });

            CANISTER_DATA.with(|canister_data_ref_cell| {
                let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
                canister_data_ref_cell
                    .post_principal_map
                    .insert((post_id.clone(), StablePrincipal(bet_maker.clone())), ());
            });
        });

        let room_details_v1 = RoomDetailsV1 {
            bet_outcome: room_details.bet_outcome.clone(),
            room_bets_total_pot: room_details.room_bets_total_pot,
            total_hot_bets: room_details.total_hot_bets,
            total_not_bets: room_details.total_not_bets,
        };

        CANISTER_DATA.with(|canister_data_ref_cell| {
            let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
            canister_data_ref_cell
                .room_details_map
                .insert(global_room_id.clone(), room_details_v1);
        });
    });
}
