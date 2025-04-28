use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA, PUMP_N_DUMP, SNAPSHOT_DATA};
use ic_cdk_macros::{query, update};
use shared_utils::common::utils::permissions::is_reclaim_canister_id;

use super::{CanisterBackupSnapshot, CanisterDataForSnapshot, TokenBetGameForSnapshot};

#[update(guard = "is_reclaim_canister_id")]
fn save_snapshot_json() -> u32 {
    notify_to_recharge_canister();
    let mut state_bytes = vec![];

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_snapshot =
            CanisterDataForSnapshot::from(&*canister_data_ref_cell.borrow());

        let serde_str = serde_json::to_string(&canister_data_snapshot).unwrap();
        state_bytes = serde_str.as_bytes().to_vec();
    });

    let len = state_bytes.len() as u32;

    SNAPSHOT_DATA.with(|snapshot_data_ref_cell| {
        *snapshot_data_ref_cell.borrow_mut() = state_bytes;
    });

    len
}

#[update(guard = "is_reclaim_canister_id")]
fn save_snapshot_json_v2() -> u32 {
    notify_to_recharge_canister();

    let canister_data_for_snapshot = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = &*canister_data_ref_cell.borrow();
        CanisterDataForSnapshot::from(canister_data)
    });

    let token_bet_game_for_snapshot = PUMP_N_DUMP.with(|pump_n_dump_ref_cell| {
        let pump_n_dump = &*pump_n_dump_ref_cell.borrow();
        TokenBetGameForSnapshot::from(pump_n_dump)
    });

    let canister_backup_snapshot = CanisterBackupSnapshot {
        canister_data_for_snapshot,
        token_bet_game_for_snapshot,
    };

    let serde_str = serde_json::to_string(&canister_backup_snapshot).unwrap();
    let state_bytes = serde_str.as_bytes().to_vec();

    let len = state_bytes.len() as u32;

    SNAPSHOT_DATA.with(|snapshot_data_ref_cell| {
        *snapshot_data_ref_cell.borrow_mut() = state_bytes;
    });

    len
}

#[query(guard = "is_reclaim_canister_id")]
fn download_snapshot(offset: u64, length: u64) -> Vec<u8> {
    let state_bytes = SNAPSHOT_DATA.with(|snapshot_data_ref_cell| {
        let snapshot = snapshot_data_ref_cell.borrow();

        snapshot[offset as usize..(offset + length) as usize].to_vec()
    });

    state_bytes
}

#[update(guard = "is_reclaim_canister_id")]
fn receive_and_save_snaphot(offset: u64, state_bytes: Vec<u8>) {
    notify_to_recharge_canister();
    SNAPSHOT_DATA.with(|snapshot_data_ref_cell| {
        let mut snapshot = snapshot_data_ref_cell.borrow_mut();
        // grow snapshot if needed
        if snapshot.len() < (offset + state_bytes.len() as u64) as usize {
            snapshot.resize((offset + state_bytes.len() as u64) as usize, 0);
        }
        snapshot.splice(
            offset as usize..(offset + state_bytes.len() as u64) as usize,
            state_bytes,
        );
    });
}

#[update(guard = "is_reclaim_canister_id")]
fn load_snapshot() {
    let state_bytes =
        SNAPSHOT_DATA.with(|snapshot_data_ref_cell| snapshot_data_ref_cell.borrow().clone());

    let canister_data_snapshot: CanisterDataForSnapshot =
        serde_json::from_str(std::str::from_utf8(&state_bytes).unwrap()).unwrap();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data_snapshot.into();
    });
}

#[update(guard = "is_reclaim_canister_id")]
fn load_snapshot_v2() {
    let state_bytes =
        SNAPSHOT_DATA.with(|snapshot_data_ref_cell| snapshot_data_ref_cell.borrow().clone());

    let canister_backup_snapshot: CanisterBackupSnapshot =
        serde_json::from_str(std::str::from_utf8(&state_bytes).unwrap()).unwrap();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() =
            canister_backup_snapshot.canister_data_for_snapshot.into();
    });

    PUMP_N_DUMP.with(|pump_n_dump_ref_cell| {
        *pump_n_dump_ref_cell.borrow_mut() =
            canister_backup_snapshot.token_bet_game_for_snapshot.into();
    });
}

#[update(guard = "is_reclaim_canister_id")]
fn clear_snapshot() {
    notify_to_recharge_canister();
    SNAPSHOT_DATA.with(|snapshot_data_ref_cell| {
        *snapshot_data_ref_cell.borrow_mut() = vec![];
    });
}
