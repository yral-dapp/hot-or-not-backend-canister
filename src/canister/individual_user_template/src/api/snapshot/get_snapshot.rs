use crate::{
    data_model::memory::{self, MEMORY_MANAGER},
    CANISTER_DATA, SNAPSHOT_DATA,
};
use candid::Principal;
use ic_cdk::api::stable;
use ic_cdk_macros::{query, update};
use ic_stable_structures::{memory_manager::MemoryId, writer::Writer, Memory};
use shared_utils::common::utils::permissions::is_reclaim_canister_id;
use shared_utils::constant::RECLAIM_CANISTER_PRINCIPAL_ID;

use super::CanisterDataForSnapshot;

#[update(guard = "is_reclaim_canister_id")]
fn save_snapshot_json() -> u32 {
    let mut state_bytes = vec![];

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_snapshot =
            CanisterDataForSnapshot::from(&*canister_data_ref_cell.borrow());

        let serde_str = serde_json::to_string(&canister_data_snapshot).unwrap();
        state_bytes = serde_str.as_bytes().to_vec();
    });

    let len = state_bytes.len() as u32;

    // let mut snapshot_memory = get_snapshot_memory();
    // let mut writer = Writer::new(&mut snapshot_memory, 0);
    // writer.write(&len.to_le_bytes()).unwrap();
    // writer.write(&state_bytes).unwrap();

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
    // let mut writer = Writer::new(&mut snapshot_memory, offset);
    // writer.write(&state_bytes).unwrap();

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
fn load_snapshot(length: u64) {
    // let mut state_bytes: Vec<u8> = vec![0; length as usize];
    // snapshot_memory.read(0, &mut state_bytes);

    let state_bytes =
        SNAPSHOT_DATA.with(|snapshot_data_ref_cell| snapshot_data_ref_cell.borrow().clone());

    let canister_data_snapshot: CanisterDataForSnapshot =
        serde_json::from_str(std::str::from_utf8(&state_bytes).unwrap()).unwrap();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data_snapshot.into();
    });
}

#[update(guard = "is_reclaim_canister_id")]
fn clear_snapshot() {
    SNAPSHOT_DATA.with(|snapshot_data_ref_cell| {
        *snapshot_data_ref_cell.borrow_mut() = vec![];
    });
}
