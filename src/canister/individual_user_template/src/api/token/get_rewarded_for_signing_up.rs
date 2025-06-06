use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA, PUMP_N_DUMP};
use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::token::TokenTransactions,
    common::{
        types::{
            known_principal::KnownPrincipalType,
            utility_token::token_event::{MintEvent, TokenEvent},
        },
        utils::system_time,
    },
};

#[update]
fn get_rewarded_for_signing_up() {
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

    let current_time = system_time::get_current_system_time_from_ic();

    let mut user_principal = Principal::anonymous();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data_ref = canister_data_ref_cell.borrow();
        let my_principal_id = canister_data_ref.profile.principal_id.unwrap();
        user_principal = my_principal_id;
    });

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        let onboarding_reward = pd.onboarding_reward.clone();
        pd.cents.handle_token_event(TokenEvent::Mint {
            amount: onboarding_reward.0.try_into().unwrap(),
            details: MintEvent::NewUserSignup {
                new_user_principal_id: user_principal,
            },
            timestamp: current_time,
        });
    });
}
