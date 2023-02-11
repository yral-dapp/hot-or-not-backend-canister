use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{BoundedStorable, Storable};

#[derive(CandidType, Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct StorablePrincipal(pub Principal);

impl Storable for StorablePrincipal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for StorablePrincipal {
    // * KeyTooLarge { given: 38, max: 32 }
    const MAX_SIZE: u32 = 38;
    const IS_FIXED_SIZE: bool = true;
}
