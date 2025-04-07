use candid::{CandidType, Nat};
use serde::{Deserialize, Serialize};

use crate::common::types::utility_token::token_event::{
    HotOrNotOutcomePayoutEvent, MintEvent, PumpDumpOutcomePayoutEvent, StakeEvent, TokenEvent,
    WithdrawEvent, HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE,
};

use super::token::{get_earnings_amount_from_winnings_amount, TokenTransactions};

#[derive(Serialize, Deserialize, Clone, Default, CandidType)]
pub struct CentsToken {
    net_airdrop: Nat,
    /// user balance
    balance: Nat,
    net_earnings: Nat,
}

impl CentsToken {
    pub fn withdrawable_balance(&self) -> Nat {
        if self.net_airdrop >= self.balance {
            0_u32.into()
        } else {
            self.balance.clone() - self.net_airdrop.clone()
        }
    }

    pub fn get_net_earnings(&self) -> Nat {
        self.net_earnings.clone()
    }

    pub fn get_net_airdrop(&self) -> Nat {
        self.net_airdrop.clone()
    }
}

impl TokenTransactions for CentsToken {
    fn get_current_token_balance(&self) -> u128 {
        self.balance.clone().0.try_into().unwrap()
    }

    fn handle_token_event(&mut self, token_event: TokenEvent) {
        match token_event {
            TokenEvent::Mint {
                details, amount, ..
            } => match details {
                MintEvent::NewUserSignup { .. } => {
                    self.balance += amount as u128;
                    self.net_airdrop += amount as u128;
                    self.net_earnings += amount as u128;
                }
                MintEvent::Referral { .. } => {
                    self.net_airdrop += amount as u128;
                    self.balance += amount as u128;
                    self.net_earnings += amount as u128;
                }
                MintEvent::Airdrop { amount } => {
                    self.net_airdrop += amount as u128;
                    self.balance += amount as u128;
                    self.net_earnings += amount as u128;
                }
            },
            TokenEvent::Burn => {}
            TokenEvent::Transfer { amount, .. } => {
                self.balance -= amount as u128;
            }
            TokenEvent::Receive { amount, .. } => {
                self.balance += amount as u128;
            }
            TokenEvent::Stake {
                details, amount, ..
            } => match details {
                StakeEvent::BetOnHotOrNotPost { bet_amount, .. } => {
                    self.balance -= bet_amount as u128;
                }
                StakeEvent::BetFailureRefund { bet_amount, .. } => {
                    self.balance += bet_amount as u128;
                }
                StakeEvent::BetOnPumpDump {
                    pumps,
                    dumps,
                    root_canister_id,
                } => {
                    self.balance -= amount as u128;
                }
            },
            TokenEvent::HotOrNotOutcomePayout { details, .. } => match details {
                HotOrNotOutcomePayoutEvent::CommissionFromHotOrNotBet {
                    room_pot_total_amount,
                    ..
                } => {
                    self.balance += (room_pot_total_amount
                        * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
                        / 100) as u128;
                    self.net_earnings += (room_pot_total_amount
                        * HOT_OR_NOT_BET_CREATOR_COMMISSION_PERCENTAGE
                        / 100) as u128;
                }
                HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                    winnings_amount, ..
                } => {
                    self.balance += winnings_amount as u128;
                    self.net_earnings +=
                        get_earnings_amount_from_winnings_amount(&winnings_amount) as u128;
                }
            },
            TokenEvent::Withdraw { amount, event_type } => match event_type {
                WithdrawEvent::WithdrawRequest => {
                    self.balance -= amount;
                }
                WithdrawEvent::WithdrawRequestFailed => {
                    self.balance += amount;
                }
            },
            TokenEvent::PumpDumpOutcomePayout {
                amount,
                payout_type,
            } => match payout_type {
                PumpDumpOutcomePayoutEvent::RewardFromPumpDumpGame { .. } => {
                    self.balance += amount;
                    self.net_earnings += amount;
                }
                PumpDumpOutcomePayoutEvent::CreatorRewardFromPumpDumpGame => {
                    self.balance += amount;
                    self.net_earnings += amount;
                }
            },
        }
    }
}
