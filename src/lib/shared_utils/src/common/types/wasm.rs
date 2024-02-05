use std::borrow::Cow;

use candid::CandidType;
use ciborium::de;
use serde::{Deserialize, Serialize};
use ic_stable_structures::{BoundedStorable, Storable};


#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, CandidType)]
pub enum WasmType {
    SubnetOrchestratorWasm,
    IndividualUserWasm,
    PostCacheWasm
}

impl BoundedStorable for WasmType {
    const MAX_SIZE: u32 = 100;

    const IS_FIXED_SIZE: bool = true;
}

impl Storable for WasmType {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let wasm_type: WasmType = de::from_reader(bytes.as_ref()).unwrap();
        wasm_type
    }
}