use ciborium::ser;
use ic_stable_structures::writer::Writer;

use crate::{data::memory_layout, CANISTER_DATA};

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    // * Serialize the state.
    let mut state_bytes = vec![];
    CANISTER_DATA
        .with(|canister_data_ref_cell| {
            ser::into_writer(&*canister_data_ref_cell.borrow(), &mut state_bytes)
        })
        .expect("failed to encode state");

    // * Write the length of the serialized bytes to memory, followed
    // * by the bytes themselves.
    let len = state_bytes.len() as u32;
    let mut memory = memory_layout::get_heap_data_memory();
    let mut writer = Writer::new(&mut memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap()
}
