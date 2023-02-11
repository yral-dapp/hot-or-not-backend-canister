use std::cell::RefCell;

use candid::Deserialize;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use serde::Serialize;
use shared_utils::{
    canister_specific::data_backup::types::all_user_data::AllUserData,
    common::types::storable_principal::StorablePrincipal,
};

use super::heap_data::HeapData;

thread_local! {
  static MEMORY_MANANGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Deserialize, Serialize)]
pub struct CanisterData {
    pub heap_data: HeapData,
    #[serde(skip, default = "init_user_principal_id_to_all_user_data_map")]
    pub user_principal_id_to_all_user_data_map:
        StableBTreeMap<StorablePrincipal, AllUserData, Memory>,
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            heap_data: HeapData::default(),
            user_principal_id_to_all_user_data_map: init_user_principal_id_to_all_user_data_map(),
        }
    }
}

// * Heap data memory.
const HEAP_DATA_MEMORY_ID: MemoryId = MemoryId::new(0);
pub fn get_heap_data_memory() -> Memory {
    MEMORY_MANANGER.with(|memory_manager_ref_cell| {
        memory_manager_ref_cell
            .borrow_mut()
            .get(HEAP_DATA_MEMORY_ID)
    })
}

// * User Principal ID to all user data map memory.
const USER_PRINCIPAL_ID_TO_ALL_USER_DATA_MAP_MEMORY_ID: MemoryId = MemoryId::new(1);
pub fn get_user_principal_id_to_all_user_data_map_memory() -> Memory {
    MEMORY_MANANGER.with(|memory_manager_ref_cell| {
        memory_manager_ref_cell
            .borrow_mut()
            .get(USER_PRINCIPAL_ID_TO_ALL_USER_DATA_MAP_MEMORY_ID)
    })
}
fn init_user_principal_id_to_all_user_data_map(
) -> StableBTreeMap<StorablePrincipal, AllUserData, Memory> {
    StableBTreeMap::init(get_user_principal_id_to_all_user_data_map_memory())
}
