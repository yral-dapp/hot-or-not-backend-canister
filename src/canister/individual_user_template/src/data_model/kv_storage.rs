use std::{borrow::Cow, collections::BTreeMap};

use crate::CANISTER_DATA;

use super::memory::{
    get_kv_storage_namespace_key_value_memory, get_kv_storage_namespace_memory, Memory,
};
use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::kv_storage::{
        NamespaceErrors, NamespaceForFrontend,
    },
    common::types::app_primitive_type::PostId,
};

type NamespaceId = u64;

#[derive(Serialize, Deserialize, Clone)]
pub struct Namespace {
    id: u64,
    title: String,
    namespace_owner_id: Principal,
}

impl From<Namespace> for NamespaceForFrontend {
    fn from(value: Namespace) -> Self {
        NamespaceForFrontend {
            id: value.id,
            title: value.title,
            owner_id: value.namespace_owner_id,
        }
    }
}

impl Storable for Namespace {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let namespace: Self = ciborium::de::from_reader(bytes.as_ref()).unwrap();
        namespace
    }

    const BOUND: ic_stable_structures::storable::Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

impl Namespace {
    pub fn is_value_above_the_size_limit(&self, value: &str) -> bool {
        value.len() > 400
    }

    pub fn write_key_value_pair(
        &self,
        key: String,
        value: String,
    ) -> Result<Option<String>, NamespaceErrors> {
        let namespace_key = NameSpaceKey {
            key,
            namespace_id: self.id,
        };
        if self.is_value_above_the_size_limit(&value) {
            return Err(NamespaceErrors::ValueTooBig);
        }

        let prev_value = CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .app_storage
                .namespace_key_value
                .insert(namespace_key, value)
        });

        Ok(prev_value)
    }

    pub fn delete_key_value_pair(&self, key: String) -> Option<String> {
        let namespace_key = NameSpaceKey {
            key,
            namespace_id: self.id,
        };
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .app_storage
                .namespace_key_value
                .remove(&namespace_key)
        })
    }

    pub fn write_multiple_key_value_pairs(
        &self,
        pairs: BTreeMap<String, String>,
    ) -> Result<(), NamespaceErrors> {
        let invalid_pair = pairs
            .iter()
            .any(|(_, val)| self.is_value_above_the_size_limit(val));

        if invalid_pair {
            return Err(NamespaceErrors::ValueTooBig);
        }

        CANISTER_DATA.with_borrow_mut(|canister_data| {
            pairs.into_iter().for_each(|pair| {
                let namespace_key = NameSpaceKey {
                    key: pair.0,
                    namespace_id: self.id,
                };

                canister_data
                    .app_storage
                    .namespace_key_value
                    .insert(namespace_key, pair.1);
            });
        });

        Ok(())
    }

    pub fn delete_multiple_keys(&self, keys: Vec<String>) {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            keys.into_iter().for_each(|key| {
                let namespace_key = NameSpaceKey {
                    key,
                    namespace_id: self.id,
                };
                canister_data
                    .app_storage
                    .namespace_key_value
                    .remove(&namespace_key);
            })
        })
    }

    pub fn list_keys(&self) -> Vec<String> {
        CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .app_storage
                .namespace_key_value
                .iter()
                .map(|(key, _)| key.key)
                .collect()
        })
    }

    pub fn read_key_value_pair(&self, key: String) -> Option<String> {
        let namespace_key = NameSpaceKey {
            key,
            namespace_id: self.id,
        };
        CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .app_storage
                .namespace_key_value
                .get(&namespace_key)
        })
    }

    pub fn new(namespace_owner_id: Principal, title: String) -> Result<Self, NamespaceErrors> {
        if namespace_owner_id == Principal::anonymous() {
            return Err(NamespaceErrors::Unauthorized);
        }

        let new_namespace_id = CANISTER_DATA
            .with_borrow(|canister_data| canister_data.app_storage.namespace_list.len());

        let new_namespace = Namespace {
            id: new_namespace_id,
            title,
            namespace_owner_id,
        };

        Ok(new_namespace)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NameSpaceKey {
    pub namespace_id: u64,
    pub key: String,
}

impl Storable for NameSpaceKey {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let namespace: Self = ciborium::de::from_reader(bytes.as_ref()).unwrap();
        namespace
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(Serialize, Deserialize)]
pub struct AppStorage {
    #[serde(skip, default = "_default_namespace_list")]
    namespace_list: StableBTreeMap<u64, Namespace, Memory>,
    #[serde(skip, default = "_default_namespace_key_value")]
    namespace_key_value: StableBTreeMap<NameSpaceKey, String, Memory>,
}

impl Default for AppStorage {
    fn default() -> Self {
        Self {
            namespace_list: _default_namespace_list(),
            namespace_key_value: _default_namespace_key_value(),
        }
    }
}

impl AppStorage {
    pub fn get_a_namespace(
        caller: Principal,
        namespace_uid: u64,
    ) -> Result<Namespace, NamespaceErrors> {
        CANISTER_DATA.with_borrow(|canister_data| {
            let namespace = canister_data
                .app_storage
                .namespace_list
                .get(&namespace_uid)
                .ok_or(NamespaceErrors::NamespaceNotFound)?;
            let profile_owner = canister_data
                .profile
                .principal_id
                .ok_or(NamespaceErrors::UserNotSignedUp)?;
            if caller != profile_owner && namespace.namespace_owner_id != caller {
                return Err(NamespaceErrors::Unauthorized);
            }

            Ok(namespace.clone())
        })
    }

    pub fn list_namespaces(start_index: usize, limit: usize) -> Vec<NamespaceForFrontend> {
        CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .app_storage
                .namespace_list
                .iter()
                .skip(start_index)
                .take(limit)
                .map(|v| NamespaceForFrontend::from(v.1))
                .collect()
        })
    }

    pub fn create_a_namespace(
        caller: Principal,
        title: String,
    ) -> Result<NamespaceForFrontend, NamespaceErrors> {
        let namespace = Namespace::new(caller, title)?;
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .app_storage
                .namespace_list
                .insert(namespace.id, namespace.clone())
        });
        Ok(NamespaceForFrontend::from(namespace))
    }
}

pub fn _default_namespace_list() -> StableBTreeMap<u64, Namespace, Memory> {
    ic_stable_structures::StableBTreeMap::init(get_kv_storage_namespace_memory())
}

pub fn _default_namespace_key_value() -> StableBTreeMap<NameSpaceKey, String, Memory> {
    ic_stable_structures::StableBTreeMap::init(get_kv_storage_namespace_key_value_memory())
}
