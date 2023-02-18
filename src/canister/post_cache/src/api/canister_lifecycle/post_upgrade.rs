use shared_utils::common::utils::stable_memory_serializer_deserializer;

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::pre_upgrade::BUFFER_SIZE_BYTES;

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    match stable_memory_serializer_deserializer::deserialize_from_stable_memory::<CanisterData>(
        BUFFER_SIZE_BYTES,
    ) {
        Ok(canister_data) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(e) => {
            ic_cdk::print(format!("Error: {:?}", e));
            panic!("Failed to restore canister data from stable memory");
        }
    }
}
