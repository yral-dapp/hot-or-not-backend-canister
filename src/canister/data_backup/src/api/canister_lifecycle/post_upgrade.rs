use std::time::Duration;

use ciborium::de;
use ic_stable_structures::Memory;

use crate::{
    api::well_known_principal::update_locally_stored_well_known_principals, data::memory_layout,
    CANISTER_DATA,
};

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    refetch_well_known_principals();
}

fn restore_data_from_stable_memory() {
    let heap_data = memory_layout::get_heap_data_memory();

    // * Read the length of the heap data state.
    // * Since heap can be at max 4 GiB, 4 bytes are enough to store the length.
    let mut heap_data_len_bytes = [0; 4];
    heap_data.read(0, &mut heap_data_len_bytes);
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    // * Read the canister data state.
    let mut canister_data_bytes = vec![0; heap_data_len];
    heap_data.read(4, &mut canister_data_bytes);

    // * Deserialize the canister data state.
    let canister_data =
        de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");
    CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell.borrow_mut() = canister_data;
    });
}

fn refetch_well_known_principals() {
    ic_cdk_timers::set_timer(Duration::from_nanos(10), || {
        ic_cdk::spawn(update_locally_stored_well_known_principals::update_locally_stored_well_known_principals())
    });
}
