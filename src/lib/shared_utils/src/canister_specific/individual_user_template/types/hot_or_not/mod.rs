use std::time::SystemTime;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType)]
pub enum BettingStatus {
    BettingOpen {
        started_at: SystemTime,
        number_of_participants: u8,
    },
    BettingClosed,
}

pub const MAXIMUM_NUMBER_OF_SLOTS: u8 = 48;
pub const DURATION_OF_EACH_SLOT_IN_SECONDS: u64 = 60 * 60;
pub const TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS: u64 =
    MAXIMUM_NUMBER_OF_SLOTS as u64 * DURATION_OF_EACH_SLOT_IN_SECONDS;

#[derive(CandidType)]
pub enum UserStatusForSpecificHotOrNotPost {
    NotParticipatedYet,
    AwaitingResult(BetDetail),
    ResultAnnounced(BetResult),
}

#[derive(CandidType)]
pub enum BetResult {
    Won(u64),
    Lost(u64),
    Draw(u64),
}

#[derive(CandidType)]
pub struct BetDetail {
    amount: u64,
    bet_direction: BetDirection,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum BetDirection {
    Hot,
    Not,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct HotOrNotBetId {
    pub canister_id: Principal,
    pub post_id: u64,
}
