use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        hot_or_not::{BetDirection, BetOutcomeForBetMaker, BetPayout, RoomBetPossibleOutcomes},
        post::Post,
    },
    common::utils::system_time,
};

use crate::data_model::CanisterData;

pub fn tabulate_hot_or_not_outcome_for_post_slot(
    canister_data: &mut CanisterData,
    post_id: u64,
    slot_id: u8,
) {
    let current_time = system_time::get_current_system_time_from_ic();
    let this_canister_id = ic_cdk::id();

    let post_to_tabulate_results_for = canister_data.all_created_posts.get_mut(&post_id).unwrap();
    let token_balance = &mut canister_data.my_token_balance;

    post_to_tabulate_results_for.tabulate_hot_or_not_outcome_for_slot_v1(
        &this_canister_id,
        &slot_id,
        token_balance,
        &current_time,
        &mut canister_data.room_details_map,
        &mut canister_data.bet_details_map,
    );

    inform_participants_of_outcome(post_to_tabulate_results_for, &slot_id);
}

fn inform_participants_of_outcome(post: &Post, slot_id: &u8) {
    let hot_or_not_details = post.hot_or_not_details.as_ref();

    if hot_or_not_details.is_none() {
        return;
    }

    let slot_details = hot_or_not_details.unwrap().slot_history.get(slot_id);

    if slot_details.is_none() {
        return;
    }

    for (_room_id, room_detail) in slot_details.unwrap().room_details.iter() {
        for (_participant, bet) in room_detail.bets_made.iter() {
            let bet_outcome_for_bet_maker: BetOutcomeForBetMaker = match room_detail.bet_outcome {
                RoomBetPossibleOutcomes::BetOngoing => BetOutcomeForBetMaker::AwaitingResult,
                RoomBetPossibleOutcomes::Draw => BetOutcomeForBetMaker::Draw(match bet.payout {
                    BetPayout::Calculated(amount) => amount,
                    _ => 0,
                }),
                RoomBetPossibleOutcomes::HotWon => match bet.bet_direction {
                    BetDirection::Hot => BetOutcomeForBetMaker::Won(match bet.payout {
                        BetPayout::Calculated(amount) => amount,
                        _ => 0,
                    }),
                    BetDirection::Not => BetOutcomeForBetMaker::Lost,
                },
                RoomBetPossibleOutcomes::NotWon => match bet.bet_direction {
                    BetDirection::Hot => BetOutcomeForBetMaker::Lost,
                    BetDirection::Not => BetOutcomeForBetMaker::Won(match bet.payout {
                        BetPayout::Calculated(amount) => amount,
                        _ => 0,
                    }),
                },
            };

            if bet_outcome_for_bet_maker == BetOutcomeForBetMaker::AwaitingResult {
                continue;
            }

            ic_cdk::spawn(receive_bet_winnings_when_distributed(
                bet.bet_maker_canister_id,
                post.id,
                bet_outcome_for_bet_maker,
            ));
        }
    }
}

async fn receive_bet_winnings_when_distributed(
    bet_maker_canister_id: Principal,
    post_id: u64,
    bet_outcome_for_bet_maker: BetOutcomeForBetMaker,
) {
    ic_cdk::call::<_, ()>(
        bet_maker_canister_id,
        "receive_bet_winnings_when_distributed",
        (post_id, bet_outcome_for_bet_maker),
    )
    .await
    .ok();
}
