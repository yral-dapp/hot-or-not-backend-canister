use std::time::SystemTime;

use candid::Principal;
use ic_cdk::api::call::CallResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetOutcomeForBetMaker, BettingStatus, HotOrNotGame, PlacedBetDetail},
        token::TokenTransactions,
    },
    common::{
        types::utility_token::token_event::{HotOrNotOutcomePayoutEvent, StakeEvent, TokenEvent},
        utils::system_time::get_current_system_time,
    },
};

use super::{pump_n_dump::TokenBetGame, CanisterData};

pub struct CentsHotOrNotGame<'a> {
    pub token_bet_game: &'a mut TokenBetGame,
    pub canister_data: &'a mut CanisterData,
}

impl<'a> HotOrNotGame for CentsHotOrNotGame<'a> {
    fn prepare_for_bet(
        &mut self,
        bet_marker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
        current_timestamp: SystemTime,
    ) -> Result<(), BetOnCurrentlyViewingPostError> {
        self.validate_incoming_bet(bet_marker_principal, &place_bet_arg)?;

        self.token_bet_game
            .cents
            .handle_token_event(TokenEvent::Stake {
                amount: place_bet_arg.bet_amount,
                details: StakeEvent::BetOnHotOrNotPost {
                    post_canister_id: place_bet_arg.post_canister_id,
                    post_id: place_bet_arg.post_id,
                    bet_amount: place_bet_arg.bet_amount,
                    bet_direction: place_bet_arg.bet_direction,
                },
                timestamp: current_timestamp,
            });

        Ok(())
    }
    fn process_place_bet_status(
        &mut self,
        bet_response: CallResult<(Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>,
        place_bet_arg: &PlaceBetArg,
        current_timestamp: SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let bet_response = bet_response
            .map_err(|_e| BetOnCurrentlyViewingPostError::PostCreatorCanisterCallFailed)
            .map(|res| res.0)
            .and_then(|inner| inner)
            .inspect_err(|_| {
                self.token_bet_game
                    .cents
                    .handle_token_event(TokenEvent::Stake {
                        amount: place_bet_arg.bet_amount,
                        details: StakeEvent::BetFailureRefund {
                            bet_amount: place_bet_arg.bet_amount,
                            post_id: place_bet_arg.post_id,
                            post_canister_id: place_bet_arg.post_canister_id,
                            bet_direction: place_bet_arg.bet_direction,
                        },
                        timestamp: current_timestamp,
                    });
            })?;

        match bet_response {
            BettingStatus::BettingClosed => {
                self.token_bet_game
                    .cents
                    .handle_token_event(TokenEvent::Stake {
                        amount: place_bet_arg.bet_amount,
                        details: StakeEvent::BetFailureRefund {
                            bet_amount: place_bet_arg.bet_amount,
                            post_id: place_bet_arg.post_id,
                            post_canister_id: place_bet_arg.post_canister_id,
                            bet_direction: place_bet_arg.bet_direction,
                        },
                        timestamp: get_current_system_time(),
                    });
                return Err(BetOnCurrentlyViewingPostError::BettingClosed);
            }
            BettingStatus::BettingOpen {
                ongoing_slot,
                ongoing_room,
                ..
            } => {
                let all_hot_or_not_bets_placed = &mut self
                    .token_bet_game
                    .hot_or_not_bet_details
                    .all_hot_or_not_bets_placed;
                all_hot_or_not_bets_placed.insert(
                    (place_bet_arg.post_canister_id, place_bet_arg.post_id),
                    PlacedBetDetail {
                        canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        slot_id: ongoing_slot,
                        room_id: ongoing_room,
                        bet_direction: place_bet_arg.bet_direction,
                        bet_placed_at: current_timestamp,
                        amount_bet: place_bet_arg.bet_amount,
                        outcome_received: BetOutcomeForBetMaker::default(),
                    },
                );
            }
        }

        Ok(bet_response)
    }

    fn validate_incoming_bet(
        &self,
        bet_maker_principal: Principal,
        place_bet_arg: &PlaceBetArg,
    ) -> Result<(), BetOnCurrentlyViewingPostError> {
        if bet_maker_principal == Principal::anonymous() {
            return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
        }

        let profile_owner = self
            .canister_data
            .profile
            .principal_id
            .ok_or(BetOnCurrentlyViewingPostError::UserPrincipalNotSet)?;

        if bet_maker_principal != profile_owner {
            return Err(BetOnCurrentlyViewingPostError::Unauthorized);
        }

        let utlility_token_balance = self.token_bet_game.cents.get_current_token_balance();

        if utlility_token_balance < place_bet_arg.bet_amount as u128 {
            return Err(BetOnCurrentlyViewingPostError::InsufficientBalance);
        }

        if self
            .token_bet_game
            .hot_or_not_bet_details
            .all_hot_or_not_bets_placed
            .contains_key(&(place_bet_arg.post_canister_id, place_bet_arg.post_id))
        {
            return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
        }

        Ok(())
    }

    fn receive_bet_from_bet_maker_canister(
        &mut self,
        bet_maker_principal_id: Principal,
        bet_maker_canister_id: Principal,
        place_bet_arg: &PlaceBetArg,
        current_timestamp: SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        let PlaceBetArg { post_id, .. } = place_bet_arg;
        self.token_bet_game.register_hot_or_not_bet_for_post_v1(
            *post_id,
            bet_maker_principal_id,
            bet_maker_canister_id,
            place_bet_arg,
            &current_timestamp,
        )
    }

    fn tabulate_hot_or_not_outcome_for_post_slot(
        &mut self,
        post_id: u64,
        slot_id: u8,
        current_timestamp: SystemTime,
    ) {
        let post_res =
            CanisterData::get_post_mut(&mut self.canister_data.all_created_posts, post_id);

        let Some(post) = post_res else {
            return;
        };

        post.tabulate_hot_or_not_outcome_for_slot_v1(
            &ic_cdk::id(),
            &slot_id,
            &mut self.token_bet_game.cents,
            &current_timestamp,
            &mut self.token_bet_game.hot_or_not_bet_details.room_details_map,
            &mut self.token_bet_game.hot_or_not_bet_details.bet_details_map,
        );

        self.canister_data
            .all_created_posts
            .get_mut(&post_id)
            .map(|post| post.slots_left_to_be_computed.remove(&slot_id));
    }

    fn receive_earnings_for_the_bet(
        &mut self,
        post_id: u64,
        post_creator_canister_id: Principal,
        outcome: BetOutcomeForBetMaker,
        current_timestamp: SystemTime,
    ) {
        if !self
            .token_bet_game
            .hot_or_not_bet_details
            .all_hot_or_not_bets_placed
            .contains_key(&(post_creator_canister_id, post_id))
        {
            return;
        }

        if self
            .token_bet_game
            .hot_or_not_bet_details
            .all_hot_or_not_bets_placed
            .get(&(post_creator_canister_id, post_id))
            .unwrap()
            .outcome_received
            != BetOutcomeForBetMaker::AwaitingResult
        {
            return;
        }

        let all_hot_or_not_bets_placed = &mut self
            .token_bet_game
            .hot_or_not_bet_details
            .all_hot_or_not_bets_placed;

        all_hot_or_not_bets_placed
            .entry((post_creator_canister_id, post_id))
            .and_modify(|placed_bet_detail| {
                placed_bet_detail.outcome_received = outcome.clone();
            });

        let placed_bet_detail = all_hot_or_not_bets_placed
            .get(&(post_creator_canister_id, post_id))
            .cloned()
            .unwrap();

        self.token_bet_game
            .cents
            .handle_token_event(TokenEvent::HotOrNotOutcomePayout {
                amount: match outcome {
                    BetOutcomeForBetMaker::Draw(amount) => amount,
                    BetOutcomeForBetMaker::Won(amount) => amount,
                    _ => 0,
                },
                details: HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                    post_canister_id: post_creator_canister_id,
                    post_id,
                    slot_id: placed_bet_detail.slot_id,
                    room_id: placed_bet_detail.room_id,
                    winnings_amount: match outcome {
                        BetOutcomeForBetMaker::Draw(amount) => amount,
                        BetOutcomeForBetMaker::Won(amount) => amount,
                        _ => 0,
                    },
                    event_outcome: outcome,
                },
                timestamp: current_timestamp,
            });
    }
}
