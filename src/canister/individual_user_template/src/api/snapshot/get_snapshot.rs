use crate::{
    data_model::{
        memory::{self, get_snapshot_memory, MEMORY_MANAGER},
        CanisterDataForSnapshot,
    },
    CANISTER_DATA,
};
use candid::Principal;
use ic_cdk::api::stable;
use ic_stable_structures::{memory_manager::MemoryId, writer::Writer, Memory};
use shared_utils::constant::RECLAIM_CANISTER_PRINCIPAL_ID;

pub fn is_reclaim_canister_id() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let reclaim_canister_principal = Principal::from_text(RECLAIM_CANISTER_PRINCIPAL_ID).unwrap();

    // Here accessing the args ???

    if caller == reclaim_canister_principal {
        Ok(())
    } else {
        Err("Caller is not allowed.".to_string())
    }
}

#[ic_cdk::update(guard = "is_reclaim_canister_id")]
#[candid::candid_method(update)]
fn save_snapshot_json() -> u32 {
    let mut state_bytes = vec![];

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_snapshot =
            CanisterDataForSnapshot::from(&*canister_data_ref_cell.borrow());

        let serde_str = serde_json::to_string(&canister_data_snapshot).unwrap();
        state_bytes = serde_str.as_bytes().to_vec();
    });

    let len = state_bytes.len() as u32;

    let mut snapshot_memory = get_snapshot_memory();
    let mut writer = Writer::new(&mut snapshot_memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();

    len
}

#[ic_cdk::query(guard = "is_reclaim_canister_id")]
#[candid::candid_method(query)]
fn download_snapshot(offset: u64, length: u64) -> Vec<u8> {
    let snapshot_memory = MEMORY_MANAGER.with(|m| m.borrow_mut().get(MemoryId::new(5)));

    let mut state_bytes = vec![0; length as usize];

    snapshot_memory.read(offset, &mut state_bytes);

    state_bytes
}

#[ic_cdk::update(guard = "is_reclaim_canister_id")]
#[candid::candid_method(update)]
fn receive_and_save_snaphot(offset: u64, state_bytes: Vec<u8>) {
    let mut snapshot_memory = get_snapshot_memory();

    let mut writer = Writer::new(&mut snapshot_memory, offset);
    writer.write(&state_bytes).unwrap();
}

#[ic_cdk::update(guard = "is_reclaim_canister_id")]
#[candid::candid_method(update)]
fn load_snapshot(length: u64) {
    let snapshot_memory = get_snapshot_memory();

    let mut state_bytes: Vec<u8> = vec![0; length as usize];
    snapshot_memory.read(0, &mut state_bytes);

    let canister_data_snapshot: CanisterDataForSnapshot =
        serde_json::from_str(std::str::from_utf8(&state_bytes).unwrap()).unwrap();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data_snapshot.into();
    });
}
