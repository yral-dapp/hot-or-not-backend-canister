use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::hot_or_not::{
    BetDetails, StablePrincipal,
};

use crate::CANISTER_DATA;

#[deprecated(note = "use get_bet_details_for_a_user_on_a_post_v1")]
#[query]
pub fn get_bet_details_for_a_user_on_a_post(
    user_principal: Principal,
    post_id: u64,
) -> Result<BetDetails, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let room_id = canister_data
            .bet_details_map
            .iter()
            .find(|(global_bet_id, bet_details)| {
                global_bet_id.0 .0 == post_id
                    && global_bet_id.1.eq(&StablePrincipal(user_principal))
            })
            .map(|v| v.1);

        room_id.ok_or(String::from("No Bets Found"))
    })
}

#[query]
pub fn get_bet_details_for_a_user_on_a_post_v1(
    user_principal: Principal,
    post_id: u64,
) -> Result<BetDetails, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let room_id = canister_data
            .hot_or_not_bet_details
            .bet_details_map
            .iter()
            .find(|(global_bet_id, bet_details)| {
                global_bet_id.0 .0 == post_id
                    && global_bet_id.1.eq(&StablePrincipal(user_principal))
            })
            .map(|v| v.1);

        room_id.ok_or(String::from("No Bets Found"))
    })
}
