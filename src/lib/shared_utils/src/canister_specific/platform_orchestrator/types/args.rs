use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::common::types::wasm::WasmType;

#[derive(CandidType, Deserialize)]
pub struct PlatformOrchestratorInitArgs {
    pub version: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct UpgradeCanisterArg {
    pub canister: WasmType,
    pub version: String,
    pub wasm_blob: Vec<u8>,
}
