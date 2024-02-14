use crate::CANISTER_DATA;
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

    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let my_token_balance = &mut canister_data_ref_cell.borrow_mut().my_token_balance;

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
    });
}
