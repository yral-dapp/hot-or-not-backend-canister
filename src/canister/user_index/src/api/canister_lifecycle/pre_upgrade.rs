use std::io::Write;

use ic_stable_structures::writer::Writer;
use shared_utils::common::utils::stable_memory_serializer_deserializer;
use ciborium::ser;
use crate::{data_model::memory, CANISTER_DATA};

pub const BUFFER_SIZE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();
        stable_memory_serializer_deserializer::serialize_to_stable_memory(
            canister_data,
            BUFFER_SIZE_BYTES,
        )
        .expect("Failed to serialize canister data");
    });

    let mut state_bytes = vec![];
    CANISTER_DATA.with_borrow(|canister_data| 
        ser::into_writer(&*canister_data, &mut state_bytes)
    )
    .expect("failed to encode state");
    let len = state_bytes.len() as u32;

    let mut upgrade_memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut upgrade_memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
    
}
