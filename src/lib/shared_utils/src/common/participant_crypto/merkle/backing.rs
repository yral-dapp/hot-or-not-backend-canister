use std::collections::{btree_map::Entry, BTreeMap};

use hash_db::{AsHashDB, HashDB, HashDBRef, Hasher, Prefix};
use memory_db::{HashKey, KeyFunction};
use serde::{Deserialize, Serialize};
use trie_db::{TrieLayout, NodeCodec};

use super::{Blake3Hasher, ChildreenTreeLayout, Hash};

type ChildrenHK = HashKey<Blake3Hasher>;

// https://docs.rs/memory-db/0.32.0/src/memory_db/lib.rs.html#83-92 modified for our uses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildrenBacking {
    data: BTreeMap<Hash, (Vec<u8>, i32)>,
    hashed_null_node: Hash,
    null_node_data: Vec<u8>,
}

impl Default for ChildrenBacking {
    fn default() -> Self {
        Self {
            data: BTreeMap::new(),
            hashed_null_node: <ChildreenTreeLayout as TrieLayout>::Codec::hashed_null_node(),
            null_node_data: <ChildreenTreeLayout as TrieLayout>::Codec::empty_node().to_vec(),
        }
    }
}

impl HashDB<Blake3Hasher, Vec<u8>> for ChildrenBacking {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        if key == &self.hashed_null_node {
            return Some(self.null_node_data.clone());
        }

        let key = ChildrenHK::key(key, prefix);
        match self.data.get(&key) {
            Some(&(ref v, rc)) if rc > 0 => Some(v.clone()),
            _ => None,
        }
    }

    fn contains(&self, key: &Hash, prefix: Prefix) -> bool {
        if key == &self.hashed_null_node {
            return true;
        }

        let key = ChildrenHK::key(key, prefix);
        self.data.get(&key).map(|&(_, rc)| rc > 0).unwrap_or_default()
    }

    fn emplace(&mut self, key: Hash, prefix: Prefix, value: Vec<u8>) {
        if value == self.null_node_data {
            return;
        }

        let key = ChildrenHK::key(&key, prefix);
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
                if *rc <= 0 {
                    *old_value = value;
                }
                *rc += 1;
            },
            Entry::Vacant(entry) => {
                entry.insert((value, 1));
            }
        }
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> Hash {
        if value == self.null_node_data {
            return self.hashed_null_node;
        }

        let key = Blake3Hasher::hash(value);
        self.emplace(key, prefix, value.to_vec());
        key
    }

    fn remove(&mut self, key: &Hash, prefix: Prefix) {
        if key == &self.hashed_null_node {
            return;
        }

        let key = ChildrenHK::key(key, prefix);
        self.data.entry(key)
            .and_modify(|&mut (_, ref mut rc)| *rc -= 1)
            .or_insert_with(|| (vec![], -1));
    }
}

impl AsHashDB<Blake3Hasher, Vec<u8>> for ChildrenBacking {
    fn as_hash_db(&self) -> &dyn HashDB<Blake3Hasher, Vec<u8>> {
        self
    }

    fn as_hash_db_mut<'a>(&'a mut self) -> &'a mut (dyn HashDB<Blake3Hasher, Vec<u8>> + 'a) {
        self
    }
}

impl HashDBRef<Blake3Hasher, Vec<u8>> for ChildrenBacking {
    fn get(&self, key: &Hash, prefix: Prefix) -> Option<Vec<u8>> {
        HashDB::get(self, key, prefix)
    }

    fn contains(&self, key: &Hash, prefix: Prefix) -> bool {
       HashDB::contains(self, key, prefix) 
    }
}

