use std::borrow::Cow;

use candid::CandidType;
use serde::{Deserialize, Serialize};
use ic_stable_structures::{storable::Bound, Storable};


#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, CandidType)]
pub enum WasmType {
    SubnetOrchestratorWasm,
    IndividualUserWasm,
    PostCacheWasm
}

impl Storable for WasmType {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = minicbor_serde::to_vec(self).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let wasm_type: WasmType = minicbor_serde::from_slice(bytes.as_ref()).unwrap();
        wasm_type
    }

    const BOUND: Bound = Bound::Bounded { max_size: 25, is_fixed_size: false };
}


#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct CanisterWasm {
    pub wasm_blob: Vec<u8>,
    pub version: String,
}

impl Storable for CanisterWasm {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let bytes = minicbor_serde::to_vec(self).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let canister_wasm: CanisterWasm = minicbor_serde::from_slice(bytes.as_ref()).unwrap();
        canister_wasm
    }

    const BOUND: Bound = Bound::Unbounded;
}
