use ic_cdk_macros::pre_upgrade;
use shared_utils::common::utils::stable_memory_serializer_deserializer;

use crate::CANISTER_DATA;

pub const BUFFER_SIZE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB

#[pre_upgrade]
fn pre_upgrade() {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();
        stable_memory_serializer_deserializer::serialize_to_stable_memory(
            canister_data,
            BUFFER_SIZE_BYTES,
        )
        .expect("Failed to serialize canister data");
    });
}
