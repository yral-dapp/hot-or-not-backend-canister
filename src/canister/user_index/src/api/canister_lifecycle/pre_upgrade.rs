use ic_cdk_macros::pre_upgrade;
use ic_stable_structures::writer::Writer;
use crate::{data_model::memory, CANISTER_DATA};


#[pre_upgrade]
fn pre_upgrade() {
    let state_bytes = CANISTER_DATA.with_borrow(|canister_data| 
        minicbor_serde::to_vec(canister_data)
    )
    .expect("failed to encode state");
    let len = state_bytes.len() as u32;

    let mut upgrade_memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut upgrade_memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
    
}
