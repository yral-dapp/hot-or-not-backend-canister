
use backing::ChildrenBacking;
use candid::{CandidType, Principal};
use generic_array::{typenum::U32, GenericArray};
use hash256_std_hasher::Hash256StdHasher;
use hash_db::Hasher;
use serde::{Deserialize, Serialize};
use trie_db::{proof::{generate_proof, verify_proof}, NodeCodec, TrieDBMut, TrieDBMutBuilder, TrieLayout, TrieMut};

use super::ProofOfChildren;

mod layout;
mod backing;

// We use GenericArray instead of [u8; 32] because serde::Serialize generates an implementation
// that is too complex for IC to run...
#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Hash(GenericArray<u8, U32>);

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Hash {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl CandidType for Hash {
    fn _ty() -> candid::types::Type {
        <[u8; 32]>::_ty()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
        where
            S: candid::types::Serializer {
        self.0.idl_serialize(serializer)
    }
}

pub(super) type ProofOfInclusion = Vec<Vec<u8>>;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Blake3Hasher;

impl Hasher for Blake3Hasher {
    type Out = Hash;

    type StdHasher = Hash256StdHasher;

    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(x);
        let hs: [u8; 32] = hasher.finalize().into();
        Hash(hs.into())
    }
}

pub(super) type ChildreenTreeLayout = layout::LayoutV1<Blake3Hasher>;

#[derive(Serialize, Deserialize)]
pub struct ChildrenMerkle {
    db: ChildrenBacking,
    pub(super) root: Hash,
    pub(super) proof_of_children: Option<ProofOfChildren>,
}

impl Default for ChildrenMerkle {
    fn default() -> Self {
        Self {
            db: ChildrenBacking::default(),
            root: <ChildreenTreeLayout as TrieLayout>::Codec::hashed_null_node(),
            proof_of_children: None,
        }
    }
}

impl ChildrenMerkle {
    fn trie_mut(&mut self) -> TrieDBMut<'_, ChildreenTreeLayout> {
        TrieDBMutBuilder::from_existing(
            &mut self.db,
            &mut self.root,
        ).build()
    }

    pub fn insert_children(&mut self, children: impl IntoIterator<Item = Principal>) {
        let prev_root = self.root;
        let mut trie = self.trie_mut();
        for child in children {
            let key = Blake3Hasher::hash(child.as_slice());
            trie.insert(key.as_ref(), b"_").expect("insertion should not fail");
        }
        std::mem::drop(trie);
        if self.root != prev_root {
            // mark proof of children as stale
            self.proof_of_children = None;
        }
    }

    pub fn remove_child(&mut self, child: Principal) {
        let mut trie = self.trie_mut();
        let key = Blake3Hasher::hash(child.as_slice());
        trie.remove(key.as_ref()).expect("removal should not fail");
        std::mem::drop(trie);

        // mark proof of children as stale
        self.proof_of_children = None;
    }

    pub fn proof_of_inclusion(&self, child: Principal) -> Result<ProofOfInclusion, String> {
        let key = Blake3Hasher::hash(child.as_slice());
        generate_proof::<_, ChildreenTreeLayout, _, _>(
            &self.db,
            &self.root,
            [&key],
        ).map_err(|e| format!("failed to generate proof of inclusion {e:?}"))
    }

    pub fn verify_proof_of_inclusion(root: Hash, proof_of_inclusion: &[Vec<u8>], child: Principal) -> Result<(), String> {
        let key = Blake3Hasher::hash(child.as_slice());
        verify_proof::<ChildreenTreeLayout, _, _, _>(
            &root,
            proof_of_inclusion,
            [&(key, Some(b"_"))]
        ).map_err(|_| "invalid proof of inclusion".to_string())
    }
}
