use candid::Principal;
use ic_cdk::api::canister_balance;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{
        BetDirection, BetMakerInformedStatus, BetOutcomeForBetMaker, BetPayout, GlobalBetId,
        GlobalRoomId, HotOrNotGame, RoomBetPossibleOutcomes,
    },
    common::utils::{system_time::get_current_system_time, task::run_task_concurrently},
};

use crate::{
    data_model::cents_hot_or_not_game::{self, CentsHotOrNotGame},
    util::cycles::request_cycles_from_subnet_orchestrator,
    CANISTER_DATA, PUMP_N_DUMP,
};

async fn recharge_based_on_number_of_bets_placed(total_bets_placed: u64) {
    let cycles = 10_000_000_000 * total_bets_placed;
    let res = request_cycles_from_subnet_orchestrator(cycles as u128).await;
    if let Err(e) = res {
        ic_cdk::println!(
            "Request cycles from subnet orchestrator failed. Error {}",
            e
        );
    }
}

pub async fn tabulate_hot_or_not_outcome_for_post_slot(post_id: u64, slot_id: u8) {
    ic_cdk::println!("Computing outcome for post:{post_id} and slot:{slot_id} ");

    let total_bets_placed_in_the_slot = CANISTER_DATA.with_borrow(|canister_data| {
        let start_global_room_id = GlobalRoomId(post_id, slot_id, 1);
        let end_global_room_id = GlobalRoomId(post_id, slot_id + 1, 1);
        canister_data
            .room_details_map
            .range(start_global_room_id..end_global_room_id)
            .fold(0_u64, |acc, (_, room_details)| {
                let total_bets_in_the_room =
                    room_details.total_hot_bets + room_details.total_not_bets;
                acc + total_bets_in_the_room
            })
    });

    recharge_based_on_number_of_bets_placed(total_bets_placed_in_the_slot).await;

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let current_timestamp = get_current_system_time();
        canister_data.tabulate_hot_or_not_outcome_for_post_slot(
            post_id,
            slot_id,
            current_timestamp,
        );
    });

    ic_cdk::println!("Computed outcome for post:{post_id} and slot:{slot_id}");

    inform_participants_of_outcome(post_id, slot_id).await;
}

pub async fn tabulate_hot_or_not_outcome_for_post_slot_v1(post_id: u64, slot_id: u8) {
    ic_cdk::println!(
        "Computing outcome for bets placed on post:{post_id} and slot:{slot_id} using cents"
    );

    let total_bets_placed_in_the_slot = CANISTER_DATA.with_borrow(|canister_data| {
        let start_global_room_id = GlobalRoomId(post_id, slot_id, 1);
        let end_global_room_id = GlobalRoomId(post_id, slot_id + 1, 1);
        canister_data
            .room_details_map
            .range(start_global_room_id..end_global_room_id)
            .fold(0_u64, |acc, (_, room_details)| {
                let total_bets_in_the_room =
                    room_details.total_hot_bets + room_details.total_not_bets;
                acc + total_bets_in_the_room
            })
    });

    recharge_based_on_number_of_bets_placed(total_bets_placed_in_the_slot).await;

    PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let current_timestamp = get_current_system_time();
            let mut cents_hot_or_not_game = CentsHotOrNotGame {
                canister_data,
                token_bet_game,
            };
            cents_hot_or_not_game.tabulate_hot_or_not_outcome_for_post_slot(
                post_id,
                slot_id,
                current_timestamp,
            )
        })
    });

    ic_cdk::println!("Computed outcome for post:{post_id} and slot:{slot_id}");

    inform_participants_of_outcome_v1(post_id, slot_id).await;
}

pub async fn inform_participants_of_outcome(post_id: u64, slot_id: u8) {
    ic_cdk::println!("Informating participant for post: {post_id} and slot: {slot_id}");
    let Some(post) = CANISTER_DATA.with_borrow(|canister_data| {
        let post = canister_data.get_post(&post_id);
        post.cloned()
    }) else {
        return;
    };

    let start_global_room_id = GlobalRoomId(post.id, slot_id, 1);
    let end_global_room_id = GlobalRoomId(post.id, slot_id + 1, 1);

    let room_details = CANISTER_DATA.with_borrow(|canister_data| {
        let room_details = canister_data
            .room_details_map
            .range(start_global_room_id..end_global_room_id)
            .collect::<Vec<_>>();

        room_details
    });

    let inform_bet_participants_grouped_by_room_futures: Vec<_> = room_details
        .iter()
        .map(|(global_room_id, room_detail)| {
            let bet_details = CANISTER_DATA.with_borrow(|canister_data| {
                canister_data
                    .bet_details_map
                    .iter()
                    .filter(|(global_bet_id, _bet_details)| global_bet_id.0 == *global_room_id)
                    .collect::<Vec<_>>()
            });

            let mut inform_bet_participants_grouped_by_room_futures = vec![];

            for (global_bet_id, bet) in bet_details.iter() {
                let bet_outcome_for_bet_maker: BetOutcomeForBetMaker = match room_detail.bet_outcome
                {
                    RoomBetPossibleOutcomes::BetOngoing => BetOutcomeForBetMaker::AwaitingResult,
                    RoomBetPossibleOutcomes::Draw => {
                        BetOutcomeForBetMaker::Draw(match bet.payout {
                            BetPayout::Calculated(amount) => amount,
                            _ => 0,
                        })
                    }
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

                inform_bet_participants_grouped_by_room_futures.push(
                    receive_bet_winnings_when_distributed(
                        global_bet_id.clone(),
                        bet.bet_maker_canister_id,
                        post.id,
                        bet_outcome_for_bet_maker,
                    ),
                );
            }
            inform_bet_participants_grouped_by_room_futures
        })
        .collect();

    let inform_bet_participants_futures: Vec<_> = inform_bet_participants_grouped_by_room_futures
        .into_iter()
        .flatten()
        .collect();

    run_task_concurrently(
        inform_bet_participants_futures.into_iter(),
        10,
        |_| {},
        || false,
    )
    .await;
}

async fn receive_bet_winnings_when_distributed(
    global_bet_id: GlobalBetId,
    bet_maker_canister_id: Principal,
    post_id: u64,
    bet_outcome_for_bet_maker: BetOutcomeForBetMaker,
) {
    ic_cdk::println!(
        "Informing participant with canister:{} for post:{post_id}",
        bet_maker_canister_id.to_string()
    );

    ic_cdk::println!("CANISTER BALANCE {}", canister_balance());

    let res = ic_cdk::call::<_, ()>(
        bet_maker_canister_id,
        "receive_bet_winnings_when_distributed",
        (post_id, bet_outcome_for_bet_maker),
    )
    .await;

    let mut bet_maker_informed_status = Some(BetMakerInformedStatus::InformedSuccessfully);

    if let Err(e) = res {
        bet_maker_informed_status = Some(BetMakerInformedStatus::Failed(format!(
            "Informing bet maker canister {} failed: {}",
            bet_maker_canister_id, e.1
        )));
        ic_cdk::println!(
            "Informing bet maker canister {} failed {:?}",
            bet_maker_canister_id,
            e.1
        );
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let bet_details_option = canister_data.bet_details_map.get(&global_bet_id);
        bet_details_option.map(|mut bet_detail| {
            bet_detail.bet_maker_informed_status = bet_maker_informed_status;
            canister_data
                .bet_details_map
                .insert(global_bet_id, bet_detail);
        });
    });
}

pub async fn inform_participants_of_outcome_v1(post_id: u64, slot_id: u8) {
    ic_cdk::println!("Informating participant for post: {post_id} and slot: {slot_id}");
    let Some(post) = CANISTER_DATA.with_borrow(|canister_data| {
        let post = canister_data.get_post(&post_id);
        post.cloned()
    }) else {
        return;
    };

    let start_global_room_id = GlobalRoomId(post.id, slot_id, 1);
    let end_global_room_id = GlobalRoomId(post.id, slot_id + 1, 1);

    let room_details = PUMP_N_DUMP.with_borrow(|token_bet_game| {
        let room_details = token_bet_game
            .hot_or_not_bet_details
            .room_details_map
            .range(start_global_room_id..end_global_room_id)
            .collect::<Vec<_>>();

        room_details
    });

    let inform_bet_participants_grouped_by_room_futures: Vec<_> = room_details
        .iter()
        .map(|(global_room_id, room_detail)| {
            let bet_details = PUMP_N_DUMP.with_borrow(|token_bet_game| {
                token_bet_game
                    .hot_or_not_bet_details
                    .bet_details_map
                    .iter()
                    .filter(|(global_bet_id, _bet_details)| global_bet_id.0 == *global_room_id)
                    .collect::<Vec<_>>()
            });

            let mut inform_bet_participants_grouped_by_room_futures = vec![];

            for (global_bet_id, bet) in bet_details.iter() {
                let bet_outcome_for_bet_maker: BetOutcomeForBetMaker = match room_detail.bet_outcome
                {
                    RoomBetPossibleOutcomes::BetOngoing => BetOutcomeForBetMaker::AwaitingResult,
                    RoomBetPossibleOutcomes::Draw => {
                        BetOutcomeForBetMaker::Draw(match bet.payout {
                            BetPayout::Calculated(amount) => amount,
                            _ => 0,
                        })
                    }
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

                inform_bet_participants_grouped_by_room_futures.push(
                    receive_bet_winnings_when_distributed_v1(
                        global_bet_id.clone(),
                        bet.bet_maker_canister_id,
                        post.id,
                        bet_outcome_for_bet_maker,
                    ),
                );
            }
            inform_bet_participants_grouped_by_room_futures
        })
        .collect();

    let inform_bet_participants_futures: Vec<_> = inform_bet_participants_grouped_by_room_futures
        .into_iter()
        .flatten()
        .collect();

    run_task_concurrently(
        inform_bet_participants_futures.into_iter(),
        10,
        |_| {},
        || false,
    )
    .await;
}

async fn receive_bet_winnings_when_distributed_v1(
    global_bet_id: GlobalBetId,
    bet_maker_canister_id: Principal,
    post_id: u64,
    bet_outcome_for_bet_maker: BetOutcomeForBetMaker,
) {
    ic_cdk::println!(
        "Informing participant with canister:{} for post:{post_id}",
        bet_maker_canister_id.to_string()
    );

    ic_cdk::println!("CANISTER BALANCE {}", canister_balance());

    let res = ic_cdk::call::<_, ()>(
        bet_maker_canister_id,
        "receive_bet_winnings_when_distributed_v1",
        (post_id, bet_outcome_for_bet_maker),
    )
    .await;

    let mut bet_maker_informed_status = Some(BetMakerInformedStatus::InformedSuccessfully);

    if let Err(e) = res {
        bet_maker_informed_status = Some(BetMakerInformedStatus::Failed(format!(
            "Informing bet maker canister {} failed: {}",
            bet_maker_canister_id, e.1
        )));
        ic_cdk::println!(
            "Informing bet maker canister {} failed {:?}",
            bet_maker_canister_id,
            e.1
        );
    }

    PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
        let bet_details_option = token_bet_game
            .hot_or_not_bet_details
            .bet_details_map
            .get(&global_bet_id);
        bet_details_option.map(|mut bet_detail| {
            bet_detail.bet_maker_informed_status = bet_maker_informed_status;
            token_bet_game
                .hot_or_not_bet_details
                .bet_details_map
                .insert(global_bet_id, bet_detail);
        });
    });
}
