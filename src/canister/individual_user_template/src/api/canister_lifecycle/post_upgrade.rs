use candid::Principal;
use ciborium::de;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
use std::{borrow::BorrowMut, time::Duration};

use crate::data_model::memory;

use shared_utils::canister_specific::individual_user_template::types::{
    arg::IndividualUserTemplateInitArgs,
    hot_or_not::{
        BetDirection, BetMakerPrincipal, GlobalBetId, GlobalRoomId, RoomDetailsV1, SlotDetailsV1,
        StablePrincipal,
    },
};

use crate::{
    api::{
        hot_or_not_bet::reenqueue_timers_for_pending_bet_outcomes::reenqueue_timers_for_pending_bet_outcomes,
        well_known_principal::update_locally_stored_well_known_principals,
    },
    CANISTER_DATA,
};

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    save_upgrade_args_to_memory();
    refetch_well_known_principals();
    reenqueue_timers_for_pending_bet_outcomes();
    reset_profile_owner();
}


fn reset_profile_owner() {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        if let Some(profile_owner) = canister_data.profile.principal_id {
            if profile_owner.eq(&Principal::anonymous()){
                canister_data.profile.principal_id = None;
            }
        }
    })
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
