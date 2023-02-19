use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{BoundedStorable, Storable};

use crate::canister_specific::individual_user_template::types::{post::Post, token::TokenBalance};

#[derive(CandidType, Deserialize)]
pub struct AllUserData {
    pub user_principal_id: Principal,
    pub user_canister_id: Principal,
    pub canister_data: UserOwnedCanisterData,
}

impl Storable for AllUserData {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }
}

impl BoundedStorable for AllUserData {
    // * 100 kB = 100_000 B
    const MAX_SIZE: u32 = 100_000;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(Deserialize, CandidType, Default)]
pub struct UserOwnedCanisterData {
    pub unique_user_name: String,
    pub all_created_posts: BTreeMap<u64, Post>,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: ProfileDetails,
    pub token_data: TokenBalance,
}

#[derive(Deserialize, CandidType, Default)]
pub struct ProfileDetails {
    pub display_name: Option<String>,
    pub profile_picture_url: Option<String>,
}
