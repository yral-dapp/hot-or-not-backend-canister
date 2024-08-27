use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};

use crate::canister_specific::individual_user_template::types::{
    post::Post, profile::UserProfile, token::TokenBalance,
};

#[derive(CandidType, Deserialize, Debug)]
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

    const BOUND: Bound = Bound::Bounded {
        max_size: 100_000, // 100kb
        is_fixed_size: false,
    };
}

#[derive(Deserialize, CandidType, Default, Debug)]
pub struct UserOwnedCanisterData {
    pub all_created_posts: BTreeMap<u64, Post>,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub token_data: TokenBalance,
}

#[derive(Deserialize, CandidType, Default, Debug)]
pub struct ProfileDetails {
    pub display_name: Option<String>,
    pub profile_picture_url: Option<String>,
}