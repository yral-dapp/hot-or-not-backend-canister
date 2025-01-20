use ciborium::de;
use ic_cdk::api::call::ArgDecoderConfig;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::reader::Reader;
use std::borrow::BorrowMut;

use crate::{data_model::memory, PUMP_N_DUMP};

use shared_utils::canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs;

use crate::{
    api::hot_or_not_bet::reenqueue_timers_for_pending_bet_outcomes::reenqueue_timers_for_pending_bet_outcomes,
    CANISTER_DATA,
};

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    save_upgrade_args_to_memory();
    migrate_excessive_tokens();
    reenqueue_timers_for_pending_bet_outcomes();
}

fn restore_data_from_stable_memory() {
    let heap_data = memory::get_upgrades_memory();
    let mut upgrade_reader = Reader::new(&heap_data, 0);

    let mut heap_data_len_bytes = [0; 4];
    upgrade_reader.read(&mut heap_data_len_bytes).unwrap();
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    let mut canister_data_bytes = vec![0; heap_data_len];
    upgrade_reader.read(&mut canister_data_bytes).unwrap();

    let canister_data =
        de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");
    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data;
    });

    upgrade_reader.read(&mut heap_data_len_bytes).unwrap();

    let mut pump_n_dump_data_bytes = vec![0; heap_data_len];
    upgrade_reader.read(&mut pump_n_dump_data_bytes).unwrap();

    match de::from_reader(&*pump_n_dump_data_bytes) {
        Ok(pd_data) => {
            PUMP_N_DUMP.with_borrow_mut(|pd| {
                *pd = pd_data;
            });
        },
        Err(e) => {
            ic_cdk::println!("WARN: pump and data not available during upgrade, assuming uninit {e}")
        }
    };
}

fn save_upgrade_args_to_memory() {
    let upgrade_args = ic_cdk::api::call::arg_data::<(IndividualUserTemplateInitArgs,)>(
        ArgDecoderConfig::default(),
    )
    .0;

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

fn migrate_excessive_tokens(){
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data_ref_cell = canister_data_ref_cell.borrow_mut();
        if canister_data_ref_cell.my_token_balance.utility_token_balance > 18_00_00_00_00_00_00_00_00_00 {
            canister_data_ref_cell.my_token_balance.utility_token_balance = 1000;
        }
    });
}