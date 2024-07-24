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

/// `u64` = number of milliseconds that have elapsed since UNIX_EPOCH
/// # Note
///
/// The maximum value of `u64` allows this struct to represent dates up to the year 292277026596.
/// However, be cautious when performing operations that might exceed this range.
#[derive(
    Clone, Copy, CandidType, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct SystemTimeInMs(u64);

impl SystemTimeInMs {
    pub fn now() -> Self {
        let time = ic_cdk::api::time() / 1_000_000;
        SystemTimeInMs(time)
    }

    pub fn duration_since(&self, earlier: &SystemTimeInMs) -> Duration {
        if self.0 >= earlier.0 {
            Duration::from_millis(self.0 - earlier.0)
        } else {
            Duration::from_millis(0)
        }
    }

    pub fn from_system_time(system_time: SystemTime) -> Self {
        let duration = system_time
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");

        let millis = duration.as_millis() as u64;
        SystemTimeInMs(millis)
    }

    /// canister_time is in nanoseconds
    pub fn from_canister_time(canister_time: u64) -> Self {
        SystemTimeInMs(canister_time / 1_000_000)
    }

    pub fn to_system_time(&self) -> Option<SystemTime> {
        let duration = Duration::from_millis(self.0);
        UNIX_EPOCH.checked_add(duration)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<SystemTimeInMs> {
        let duration_ms = duration.as_millis() as u64;
        self.0.checked_add(duration_ms).map(SystemTimeInMs)
    }


    /// Calculates the remaining interval between the current time and a future time.
    ///
    /// The minimum returned duration is 3 seconds. This is to handle concurrent operations,
    /// specifically for processing bets on multiple posts by the same user. If the last bet timer
    /// was just processed, this ensures a 3-second wait before processing another post.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// 
    /// let current_time = SystemTimeInMs(1620);
    /// let earlier_time = SystemTimeInMs(1600);
    /// let future_duration = Duration::from_secs(3600); // 1 hour
    ///
    /// let result = current_time.calculate_remaining_interval(&earlier_time, future_duration);
    /// assert!(result.is_ok());
    /// 
    /// if let Ok(interval) = result {
    ///     assert_eq!(interval, Duration::from_secs(3580));
    ///     println!("Remaining interval: {:?}", interval);
    /// }
    ///
    /// // Test the minimum 3-second return
    /// let current_time = SystemTimeInMs(5000);
    /// let earlier_time = SystemTimeInMs(1000);
    /// let future_duration = Duration::from_secs(3); // 3 seconds
    ///
    /// let result = current_time.calculate_remaining_interval(&earlier_time, future_duration);
    /// assert!(result.is_ok());
    /// 
    /// if let Ok(interval) = result {
    ///     assert_eq!(interval, Duration::from_secs(3));
    ///     println!("Minimum interval: {:?}", interval);
    /// }
    /// ```
    pub fn calculate_remaining_interval(
        &self,
        earlier_time: &SystemTimeInMs,
        future_duration: Duration,
    ) -> Result<Duration, &'static str> {
        let future_time = earlier_time
            .checked_add(future_duration)
            .ok_or("Overflow when calculating future time")?;

        if future_time.0 > self.0 {
            Ok(Duration::from_millis(future_time.0 - self.0))
        } else {
            // keeping default to be 3s instead of 0ms for concurrent bets on two posts of the same user
            // if the last best timer was just processed, wait for 3s to process another post.
            Ok(Duration::from_millis(3000))
        }
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
        // let value = Cow::Owned(Encode!(&self.0).unwrap());
        // dbg!("  ENCODE SystemTimeInMs {} \n\n", "//".repeat(400));
        // dbg!(&value);
        // value

        // let value =
        Cow::Owned(Encode!(&self).unwrap())
        // ic_cdk::println!("value: {:?}", value);
        // value
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // Decode the bytes back into a u128 value
        // let print_val = if bytes.len() != 16 {
        //     format!("Expected 16 bytes for u128, got {}", bytes.len())
        // } else {
        //     format!("16 bytes length {} ", bytes.len())
        // };

        // ic_cdk::println!("print_val: {:?}", print_val);
        // ic_cdk::println!("bytes: {:?}", bytes);
        // let value =
        Decode!(&bytes, Self).unwrap()
        // ic_cdk::println!("print_val dfx: {:?}", value);

        // value
        // let (value, two) = Decode!(&bytes,  u128, String).unwrap();
        // ic_cdk::println!("two: {:?}", two);

        // SystemTimeInMs(value)
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100, // size of u128 in bytes
        is_fixed_size: false,
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
        // dbg!("  ENCODE NewSlotType {} \n\n", "//".repeat(400));
        Cow::Owned(Encode!(&self).unwrap())
        // Cow::Owned(Encode!(&self.0).unwrap())
    }

    fn from_bytes(from_u64_bytes: Cow<[u8]>) -> Self {
        // dbg!("DECODE NewSlotType \n\n ", "\\".repeat(10));
        // let inner_u64 = Decode!(&from_u64_bytes, u64).unwrap();
        // Self(inner_u64)
        // let inner_u64
        Decode!(&from_u64_bytes, Self).unwrap()
        // ic_cdk::println!("inner_u64 dfx: {:?}", inner_u64);
        // inner_u64
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 150,
        is_fixed_size: false,
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
