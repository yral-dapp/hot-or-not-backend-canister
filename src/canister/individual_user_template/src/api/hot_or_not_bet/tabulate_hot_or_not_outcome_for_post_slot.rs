use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        hot_or_not::{
            BetDetails, BetDirection, BetMaker, BetMakerInformedStatus, BetOutcomeForBetMaker,
            BetPayout, GlobalBetId, GlobalRoomId, RoomBetPossibleOutcomes, RoomDetailsV1,
            SlotDetails, StablePrincipal,
        },
        post::Post,
    },
    common::{types::known_principal::KnownPrincipalType, utils::system_time},
};

use crate::{
    data_model::CanisterData, util::cycles::recieve_cycles_from_subnet_orchestrator, CANISTER_DATA,
};

pub fn tabulate_hot_or_not_outcome_for_post_slot(post_id: u64, slot_id: u8) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let subnet_orchestrator_canister_id = canister_data
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .copied();

        recharge_indvidual_canister_using_subnet_orchestrator_if_needed(
            subnet_orchestrator_canister_id,
        );

        let current_time = system_time::get_current_system_time_from_ic();
        let this_canister_id = ic_cdk::id();

        let post_to_tabulate_results_for =
            canister_data.all_created_posts.get_mut(&post_id).unwrap();
        let token_balance = &mut canister_data.my_token_balance;

        post_to_tabulate_results_for.tabulate_hot_or_not_outcome_for_slot_v1(
            &this_canister_id,
            &slot_id,
            token_balance,
            &current_time,
            &mut canister_data.room_details_map,
            &mut canister_data.bet_details_map,
        );
    });

    inform_participants_of_outcome(post_id, slot_id);
}

pub fn inform_participants_of_outcome(post_id: u64, slot_id: u8) {
    let post = CANISTER_DATA.with_borrow(|canister_data| {
        let post = canister_data.all_created_posts.get(&post_id);
        post.unwrap().clone()
    });

    let start_global_room_id = GlobalRoomId(post.id, slot_id, 1);
    let end_global_room_id = GlobalRoomId(post.id, slot_id + 1, 1);

    let room_details = CANISTER_DATA.with_borrow(|canister_data| {
        let room_details = canister_data
            .room_details_map
            .range(start_global_room_id..end_global_room_id)
            .collect::<Vec<_>>();

        room_details
    });

    room_details.iter().for_each(|(_groomid, room_detail)| {
        let start_global_bet_id = GlobalBetId(start_global_room_id, StablePrincipal::default());
        let end_global_bet_id = GlobalBetId(end_global_room_id, StablePrincipal::default());
        let bet_details = CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .bet_details_map
                .range(start_global_bet_id..end_global_bet_id)
                .collect::<Vec<_>>()
        });

        for (global_bet_id, bet) in bet_details.iter() {
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
                global_bet_id.clone(),
                bet.bet_maker_canister_id,
                post.id,
                bet_outcome_for_bet_maker,
            ));
        }
    });
}

fn recharge_indvidual_canister_using_subnet_orchestrator_if_needed(
    subnet_orchestrator_canister_id: Option<Principal>,
) {
    ic_cdk::spawn(async move {
        recieve_cycles_from_subnet_orchestrator(subnet_orchestrator_canister_id).await;
    });
}

async fn receive_bet_winnings_when_distributed(
    global_bet_id: GlobalBetId,
    bet_maker_canister_id: Principal,
    post_id: u64,
    bet_outcome_for_bet_maker: BetOutcomeForBetMaker,
) {
    let res = ic_cdk::call::<_, ()>(
        bet_maker_canister_id,
        "receive_bet_winnings_when_distributed",
        (post_id, bet_outcome_for_bet_maker),
    )
    .await;

    let mut bet_maker_informed_status = Some(BetMakerInformedStatus::InformedSuccessfully);

    if let Err(e) = res {
        bet_maker_informed_status = Some(BetMakerInformedStatus::Failed(e.1));
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let bet_details_option = canister_data.bet_details_map.get(&global_bet_id);
        bet_details_option.map(|mut bet_detail| {
            bet_detail.bet_maker_informed_status = bet_maker_informed_status;
            canister_data
                .bet_details_map
                .insert(global_bet_id, bet_detail);
        });
    })
}
