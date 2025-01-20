use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA,
};
use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::{
    types::{
        known_principal::KnownPrincipalType,
        utility_token::token_event::{MintEvent, TokenEvent},
    },
    utils::system_time,
};

#[update]
fn get_rewarded_for_referral(referrer: Principal, referree: Principal) {
    // * access control
    notify_to_recharge_canister();
    let request_maker = ic_cdk::caller();
    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    if user_index_canister_principal_id != request_maker {
        return;
    }

    update_last_canister_functionality_access_time();

    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|cdata| {
        let my_token_balance = &mut cdata.my_token_balance;

        let referral_reward_amount =
            TokenEvent::get_token_amount_for_token_event(&TokenEvent::Mint {
                amount: 0,
                details: MintEvent::Referral {
                    referrer_user_principal_id: referrer,
                    referee_user_principal_id: referree,
                },
                timestamp: current_time,
            });

        my_token_balance.handle_token_event(TokenEvent::Mint {
            amount: referral_reward_amount,
            details: MintEvent::Referral {
                referrer_user_principal_id: referrer,
                referee_user_principal_id: referree,
            },
            timestamp: current_time,
        });

        cdata.pump_n_dump.game_only_balance += cdata.pump_n_dump.referral_reward.clone();
    });
}
