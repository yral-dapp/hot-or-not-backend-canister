use std::borrow::Cow;

use candid::{CandidType, Nat, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AirdropMember {
    pub user_principal: Principal,
    pub user_canister: Principal,
}

impl Storable for AirdropMember {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes)
            .expect("Expected to serialize AirdropMember");

        bytes.into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::from_reader(bytes.as_ref())
            .expect("Expected to deserialize AirdropMember")
    }
}

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub struct TokenClaim {
    pub amount: Nat,
    pub token_root: Principal,
    pub token_ledger: Principal,
    pub sender_canister: Principal,
}