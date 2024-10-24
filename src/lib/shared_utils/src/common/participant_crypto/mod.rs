//! Utilities for creating and verifying proof that a given canister is a part of YRAL Backend canisters
mod types;

use std::{cell::RefCell, collections::HashMap, thread::LocalKey};

use candid::{CandidType, Principal};
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use types::{ManagementCanisterSchnorrPublicKeyReply, ManagementCanisterSchnorrPublicKeyRequest, ManagementCanisterSignatureReply, ManagementCanisterSignatureRequest, SchnorrAlgorithm, SchnorrKeyId};
use serde::{Serialize, Deserialize};

const fn is_local() -> bool {
    let Some(network) = std::option_env!("DFX_NETWORK") else {
        return true;
    };

    match network.as_bytes() {
        b"ic" => false,
        b"local" => true,
        _ => panic!("unknown `DFX_NETWORK`"),
    }
}

pub(crate) const THRESHOLD_SCHNORR_KEY: &str = if is_local() {
    "dfx_test_key"
} else {
    "key_1"
};

pub(crate) type LocalPoPStore<Store> = LocalKey<RefCell<Store>>;

#[derive(Default, Serialize, Deserialize)]
pub struct PubKeyCache(HashMap<Principal, Vec<u8>>);

impl PubKeyCache {
    async fn get_or_init_public_key<Store: ProofOfParticipationStore>(store: &'static LocalPoPStore<Store>, principal: Principal) -> Result<VerifyingKey, String> {
        let maybe_pk = store.with_borrow(|store| {
            store.pubkey_cache().0.get(&principal).cloned()
        });
        if let Some(pk) = maybe_pk {
            return VerifyingKey::try_from(pk.as_slice())
                .map_err(|_| "invalid public key".to_string())
        }

        let derive_args = ManagementCanisterSchnorrPublicKeyRequest {
            derivation_path: vec![],
            canister_id: Some(principal),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: THRESHOLD_SCHNORR_KEY.to_string(),
            }
        };
        let (key_res,): (ManagementCanisterSchnorrPublicKeyReply,) = ic_cdk::call(
            Principal::management_canister(),
            "schnorr_public_key",
            (derive_args,)
        )
            .await
            .map_err(|(_, msg)| {
                format!("unable to get public key: {msg}")
            })?;

        let key = key_res.public_key;
        let vk = VerifyingKey::try_from(key.as_slice())
            .map_err(|_| "invalid public key".to_string())?;
        store.with_borrow_mut(|store| {
            store.pubkey_cache_mut().0.insert(principal, key.clone());
        });

        Ok(vk)
    }
}

#[derive(Serialize)]
struct ProofOfAuthorityMsg {
    prefix: &'static [u8],
    pub child: Principal,
}

impl ProofOfAuthorityMsg {
    pub fn new(child: Principal) -> Self {
        Self {
            prefix: b"CHILD",
            child,
        }
    }

    pub fn serialize_cbor(&self) -> Vec<u8> {
        let mut bytes = vec![];
        ciborium::into_writer(self, &mut bytes)
            .expect("PoaMessage should serialize succesfully");

        bytes
    }
}

/// Proof that this canister id is a child of the parent canister 
#[derive(Clone, CandidType, Serialize, Deserialize)]
struct ProofOfChild {
    // Principal of the child
    principal: Principal,
    signature: Vec<u8>,
}

impl ProofOfChild {
    async fn new(child: Principal) -> Result<Self, String> {
        let message = ProofOfAuthorityMsg::new(child);
        let sign_args = ManagementCanisterSignatureRequest {
            message: message.serialize_cbor(),
            derivation_path: vec![],
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: THRESHOLD_SCHNORR_KEY.to_string()
            }, 
        };

        let (sig_res,): (ManagementCanisterSignatureReply,) = ic_cdk::api::call::call_with_payment(
            Principal::management_canister(),
            "sign_with_schnorr",
            (sign_args,),
            25_000_000_000,
        )
        .await
        .map_err(|(_, msg)| format!("unable to sign: {msg}"))?;

        Ok(Self {
            principal: child,
            signature: sig_res.signature,
        })
    }

    pub fn verify(&self, parent_key: &VerifyingKey) -> Result<(), String> {
        let message = ProofOfAuthorityMsg::new(self.principal);
        let message_raw = message.serialize_cbor();

        let sig = Signature::from_slice(&self.signature).map_err(|_| "invalid proof".to_string())?;

        parent_key.verify(&message_raw, &sig).map_err(|_| "invalid proof".to_string())?;

        Ok(())
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct ProofOfParticipation {
    chain: Vec<ProofOfChild>,
}

impl ProofOfParticipation {
    /// New PoP for platform orchestrator
    pub fn new_for_root() -> Self {
        Self {
            chain: vec![],
        }
    } 

    pub async fn derive_for_child(self, child: Principal) -> Result<Self, String> {
        let mut chain = self.chain;
        let proof = ProofOfChild::new(child).await?;
        chain.push(proof);
        Ok(Self {
            chain,
        })
    }

    /// Verify that the caller is a YRAL canister
    pub async fn verify_caller_is_participant<Store: ProofOfParticipationStore>(&self, store: &'static LocalPoPStore<Store>) -> Result<(), String> {
        if is_local() {
            // Hack: Always pass on local testing node
            // a proper implementation requires deploying platform orchestrator locally
            return Ok(())
        }

        let platform_orchestrator = store.with_borrow(|s| s.platform_orchestrator());
        let canister = ic_cdk::caller();

        let mut parent = PubKeyCache::get_or_init_public_key(store, platform_orchestrator).await?;
        for proof in &self.chain {
            proof.verify(&parent)?;
            if proof.principal == canister {
                return Ok(())
            }
            parent = PubKeyCache::get_or_init_public_key(store, proof.principal).await?;
        }

        Err("invalid proof".to_string())
    }
}

pub trait ProofOfParticipationStore {
    fn pubkey_cache(&self) -> &PubKeyCache;

    fn pubkey_cache_mut(&mut self) -> &mut PubKeyCache;

    fn platform_orchestrator(&self) -> Principal;
}
