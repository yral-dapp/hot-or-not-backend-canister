
use backing::ChildrenBacking;
use candid::Principal;
use hash256_std_hasher::Hash256StdHasher;
use hash_db::Hasher;
use serde::{Deserialize, Serialize};
use trie_db::{proof::{generate_proof, verify_proof}, NodeCodec, TrieDBMut, TrieDBMutBuilder, TrieLayout, TrieMut};

use super::ProofOfChildren;

mod layout;
mod backing;

pub(super) type Hash = [u8; 32];
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
        hs
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
            trie.insert(&key, b"_").expect("insertion should not fail");
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
        trie.remove(&key).expect("removal should not fail");
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
