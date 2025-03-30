use ic_cdk_macros::update;

use shared_utils::{
    canister_specific::individual_user_template::types::{
        hot_or_not::{BetOutcomeForBetMaker, HotOrNotGame},
        token::TokenTransactions,
    },
    common::{
        types::{
            app_primitive_type::PostId,
            utility_token::token_event::{HotOrNotOutcomePayoutEvent, TokenEvent},
        },
        utils::system_time,
    },
};

use crate::{
    util::cycles::{notify_to_recharge_canister, recharge_canister},
    CANISTER_DATA,
};

#[update]
fn receive_bet_winnings_when_distributed(post_id: PostId, outcome: BetOutcomeForBetMaker) {
    notify_to_recharge_canister();

    let post_creator_canister_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    ic_cdk::println!(
        "Recieved bet outcome from canister {} for post {}",
        post_creator_canister_id.to_string(),
        post_id
    );

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut utility_token = canister_data.my_token_balance.clone();
        canister_data.receive_earnings_for_the_bet(
            post_id,
            post_creator_canister_id,
            outcome,
            &mut utility_token,
            current_time,
        );
        canister_data.my_token_balance = utility_token;
    })
}
