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


#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct CanisterWasm {
    pub wasm_blob: Vec<u8>,
    pub version: String,
}

impl BoundedStorable for CanisterWasm {
    const MAX_SIZE: u32 = 200_000_000; // 2 MB
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for CanisterWasm {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let canister_wasm: CanisterWasm = de::from_reader(bytes.as_ref()).unwrap();
        canister_wasm
    }
}
