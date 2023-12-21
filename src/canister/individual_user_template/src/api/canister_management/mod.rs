use ic_cdk::api::stable::stable_size;

#[candid::candid_method(query)]
#[ic_cdk::query]
pub fn get_stable_memory_size() -> u32 {
    stable_size()
}