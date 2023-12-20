use ciborium::ser;
use ic_cdk::api::stable;
use ic_stable_structures::writer::Writer;

use crate::data_model::memory;
use crate::CANISTER_DATA;


#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    let mut state_bytes = vec![];
    CANISTER_DATA.with(|canister_data_ref_cell| {
        ser::into_writer(&*canister_data_ref_cell.borrow(), &mut state_bytes)
    })
    .expect("failed to encode state");

    let len = state_bytes.len() as u32;
    
    if stable::stable_size() == 0 {
        memory::init_memory_manager();
    }
    let mut upgrade_memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut upgrade_memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
}