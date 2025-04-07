use candid::Nat;

pub mod permissions;
pub mod stable_memory_serializer_deserializer;
pub mod system_time;
pub mod task;
pub mod upgrade_canister;

#[cfg(target_arch = "wasm32")]
const WASM_PAGE_SIZE: u64 = 65536;

pub fn get_stable_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (ic_cdk::api::stable::stable_size()) * WASM_PAGE_SIZE
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn get_heap_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (core::arch::wasm32::memory_size(0) as u64) * WASM_PAGE_SIZE
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn default_pump_dump_onboarding_reward() -> Nat {
    // 1000 DOLLR
    (1e9 as u64).into()
}
