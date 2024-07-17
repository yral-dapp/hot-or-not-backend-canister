use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::canister_specific::individual_user_template::types::hot_or_not::{
    BetDirection, BetOutcomeForBetMaker,
};

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum TokenEvent {
    Mint {
        amount: u64,
        details: MintEvent,
        timestamp: SystemTime,
    },
    Burn,
    Transfer {
        amount: u64,
        to_account: Principal,
        timestamp: SystemTime,
    },
    Receive {
        amount: u64,
        from_account: Principal,
        timestamp: SystemTime,
    },
    Stake {
        amount: u64,
        details: StakeEvent,
        timestamp: SystemTime,
    },
    HotOrNotOutcomePayout {
        amount: u64,
        details: HotOrNotOutcomePayoutEvent,
        timestamp: SystemTime,
    },
}

impl TokenEvent {
    pub fn get_token_amount_for_token_event(&self) -> u64 {
        match self {
            TokenEvent::Mint { details, .. } => match details {
                MintEvent::NewUserSignup { .. } => 1000,
                MintEvent::Referral { .. } => 500,
            },
            _ => 0,
        }
    }
}

#[derive(Clone, CandidType, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum MintEvent {
    NewUserSignup {
        new_user_principal_id: Principal,
    },
    Referral {
        referee_user_principal_id: Principal,
        referrer_user_principal_id: Principal,
    },
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum StakeEvent {
    BetOnHotOrNotPost {
        post_canister_id: Principal,
        post_id: u64,
        bet_amount: u64,
        bet_direction: BetDirection,
    },
}

#[derive(
    Clone, Copy, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct SystemTimeInMs(u128);

impl SystemTimeInMs {
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        SystemTimeInMs(duration.as_millis())
    }

    // pub fn duration_since(&self, earlier: &SystemTimeInMs) -> Duration {
    //     if self.0 >= earlier.0 {
    //         Duration::from_millis((self.0 - earlier.0) as u64)
    //     } else {
    //         Duration::from_millis(0)
    //     }
    // }
    pub fn to_system_time(&self) -> Option<SystemTime> {
        let duration = Duration::from_millis(self.0 as u64);
        UNIX_EPOCH.checked_add(duration)
    }
}

impl Default for SystemTimeInMs {
    fn default() -> Self {
        SystemTimeInMs::now()
    }
}

impl Storable for SystemTimeInMs {
    fn to_bytes(&self) -> Cow<[u8]> {
        // Encode the u128 value to bytes
        Cow::Owned(Encode!(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // Decode the bytes back into a u128 value
        let value: u128 = Decode!(&bytes, u128).unwrap();
        SystemTimeInMs(value)
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 16, // size of u128 in bytes
        is_fixed_size: true,
    };
}

#[derive(
    Clone, Copy, Default, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct NewSlotType(pub u64);

impl NewSlotType {
    pub fn increment_by(&self, by: u64) -> Self {
        NewSlotType(self.0 + by)
    }
}

impl From<u8> for NewSlotType {
    fn from(value: u8) -> Self {
        NewSlotType(value as u64)
    }
}

impl Storable for NewSlotType {
    fn to_bytes(&self) -> Cow<[u8]> {
        dbg!("  ENCODE NewSlotType {} \n\n", "//".repeat(400));
        Cow::Owned(Encode!(&self.0).unwrap())
        // Cow::Owned(Encode!(&self.0).unwrap())
    }

    fn from_bytes(from_u64_bytes: Cow<[u8]>) -> Self {
        dbg!("DECODE NewSlotType \n\n ", "\\".repeat(10));
        let inner_u64 = Decode!(&from_u64_bytes, u64).unwrap();
        Self(inner_u64)
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 150,
        is_fixed_size: true,
    };
}

impl Hash for NewSlotType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum HotOrNotOutcomePayoutEvent {
    CommissionFromHotOrNotBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: NewSlotType,
        room_id: u64,
        room_pot_total_amount: u64,
    },
    WinningsEarnedFromBet {
        post_canister_id: Principal,
        post_id: u64,
        slot_id: NewSlotType,
        room_id: u64,
        event_outcome: BetOutcomeForBetMaker,
        winnings_amount: u64,
    },
}

pub const HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE: u64 = 10;
pub const HOT_OR_NOT_BET_WINNINGS_MULTIPLIER: u64 = 2;
