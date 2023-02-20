use std::collections::BTreeMap;

use candid::{CandidType, Deserialize};

use crate::types::utility_token::token_event::TokenEvent;

#[derive(Default, Clone, Deserialize, CandidType, Debug)]
pub struct TokenBalance {
    pub utility_token_balance: u64,
    pub utility_token_transaction_history_v1: BTreeMap<u64, TokenEvent>,
}

impl TokenBalance {
    pub fn get_utility_token_balance(&self) -> u64 {
        self.utility_token_balance
    }

    pub fn get_utility_token_transaction_history(&self) -> &BTreeMap<u64, TokenEvent> {
        &self.utility_token_transaction_history_v1
    }

    pub fn handle_token_event(mut self, token_event: TokenEvent) -> Self {
        self.utility_token_balance += token_event.get_token_amount_for_token_event();

        if self.utility_token_transaction_history_v1.len() > 1500 {
            self.utility_token_transaction_history_v1 = self
                .utility_token_transaction_history_v1
                .into_iter()
                .rev()
                .take(1000)
                .rev()
                .collect();
        }

        self.utility_token_transaction_history_v1.insert(
            self.utility_token_transaction_history_v1.len() as u64,
            token_event,
        );

        self
    }
}
