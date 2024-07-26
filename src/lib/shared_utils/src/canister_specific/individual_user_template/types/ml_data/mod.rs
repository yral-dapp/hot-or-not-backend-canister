use std::borrow::Cow;
use std::time::SystemTime;

use candid::CandidType;
use candid::Decode;
use candid::Deserialize;
use candid::Encode;
use candid::Principal;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Clone, CandidType)]
pub struct WatchHistoryItem {
    post_id: u64,
    publisher_canister_id: Principal,
    viewed_at: SystemTime,
    cf_video_id: String,
    percentage_watched: f32,
}

impl Ord for WatchHistoryItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.viewed_at.cmp(&other.viewed_at)
    }
}

impl Eq for WatchHistoryItem {}

impl Storable for WatchHistoryItem {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

#[derive(Deserialize, Serialize, PartialEq, PartialOrd, Clone, CandidType)]
pub struct SuccessHistoryItem {
    post_id: u64,
    publisher_canister_id: Principal,
    interacted_at: SystemTime,
    cf_video_id: String,
}

impl Ord for SuccessHistoryItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.interacted_at.cmp(&other.interacted_at)
    }
}

impl Eq for SuccessHistoryItem {}

impl Storable for SuccessHistoryItem {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}
