use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        types::utility_token::token_event::{PumpDumpOutcomePayoutEvent, StakeEvent, TokenEvent},
        utils::system_time::get_current_system_time,
    },
    constant::GDOLLR_TO_E8S,
};

#[derive(Serialize, Deserialize, Clone, Copy, CandidType, Debug, PartialEq, Eq)]
pub enum GameDirection {
    Pump,
    Dump,
}

#[derive(Serialize, Deserialize, Clone, CandidType, Debug, PartialEq, Eq)]
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
    CreatorReward(Nat),
}

impl PumpNDumpStateDiff {
    pub fn get_token_events_from_pump_dump_state_diff(&self) -> Vec<TokenEvent> {
        let mut token_events = vec![];
        match self {
            PumpNDumpStateDiff::CreatorReward(reward) => {
                let token_event = TokenEvent::PumpDumpOutcomePayout {
                    //Safety: reward is always positive and hence could be converted into u128
                    amount: reward.clone().0.try_into().unwrap(),
                    payout_type: PumpDumpOutcomePayoutEvent::CreatorRewardFromPumpDumpGame,
                };

                token_events.push(token_event);
            }
            PumpNDumpStateDiff::Participant(participated_game_info) => {
                let stake_amount =
                    (participated_game_info.pumps + participated_game_info.dumps) * GDOLLR_TO_E8S;
                token_events.push(TokenEvent::Stake {
                    amount: stake_amount,
                    details: StakeEvent::BetOnPumpDump {
                        pumps: participated_game_info.pumps,
                        dumps: participated_game_info.dumps,
                        root_canister_id: participated_game_info.token_root,
                    },
                    timestamp: get_current_system_time(),
                });

                if participated_game_info.reward > 0_u128 {
                    token_events.push(TokenEvent::PumpDumpOutcomePayout {
                        //Safety: reward is always positive and hence could be converted into u128
                        amount: participated_game_info.reward.clone().0.try_into().unwrap(),
                        payout_type: PumpDumpOutcomePayoutEvent::RewardFromPumpDumpGame {
                            game_direction: participated_game_info.game_direction,
                            token_root_canister_id: participated_game_info.token_root,
                        },
                    })
                }
            }
        }

        token_events
    }
}

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub struct PumpsAndDumps {
    pub pumps: Nat,
    pub dumps: Nat,
}

#[derive(Serialize, Deserialize, Clone, CandidType)]
pub struct BalanceInfo {
    pub net_airdrop_reward: Nat,
    pub balance: Nat,
    pub withdrawable: Nat,
}
