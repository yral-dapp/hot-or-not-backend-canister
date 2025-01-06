use candid::{CandidType, Nat, Principal};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone, Copy, CandidType)]
pub enum GameDirection {
    Pump,
    Dump,
}

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub struct ParticipatedGameInfo {
    pub pumps: u64,
    pub dumps: u64,
    pub reward: Nat,
    pub token_root: Principal,
    pub game_direction: GameDirection,
}

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub enum PumpNDumpStateDiff {
    Participant(ParticipatedGameInfo),
    CreatorReward(Nat)
}

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub struct PumpsAndDumps {
    pub pumps: Nat,
    pub dumps: Nat,
}