use candid::Principal;
use shared_utils::{
    common::types::storable_principal::StorablePrincipal, types::utility_token::v1::TokenEventV1,
};

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn receive_all_token_transactions_from_individual_user_canister(
    all_token_transactions_from_individual_user_canister_chunk: Vec<(u64, TokenEventV1)>,
    canister_owner_principal_id: Principal,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_all_token_transactions_from_individual_user_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            all_token_transactions_from_individual_user_canister_chunk,
            &caller_principal_id,
            &canister_owner_principal_id,
        );
    });
}

fn receive_all_token_transactions_from_individual_user_canister_impl(
    canister_data: &mut CanisterData,
    all_token_transactions_from_individual_user_canister_chunk: Vec<(u64, TokenEventV1)>,
    caller_principal_id: &Principal,
    canister_owner_principal_id: &Principal,
) {
    let does_the_current_call_makers_record_exist = canister_data
        .user_principal_id_to_all_user_data_map
        .contains_key(&StorablePrincipal(*canister_owner_principal_id));

    if !does_the_current_call_makers_record_exist {
        return;
    }

    let mut existing_entry = canister_data
        .user_principal_id_to_all_user_data_map
        .get(&StorablePrincipal(*canister_owner_principal_id))
        .unwrap();

    if existing_entry.user_canister_id != *caller_principal_id {
        return;
    }

    all_token_transactions_from_individual_user_canister_chunk
        .iter()
        .for_each(|token_transaction| {
            // upsert the post details in the user's record.
            existing_entry
                .canister_data
                .token_data
                .utility_token_transaction_history_v1
                .insert(token_transaction.0, token_transaction.1);
        });

    canister_data.user_principal_id_to_all_user_data_map.insert(
        StorablePrincipal(*canister_owner_principal_id),
        existing_entry,
    );
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use ic_stable_memory::utils::ic_types::SPrincipal;
    use shared_utils::{
        canister_specific::data_backup::types::all_user_data::{
            AllUserData, UserOwnedCanisterData,
        },
        types::utility_token::v0::MintEvent,
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_canister_id, get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_receive_all_token_transactions_from_individual_user_canister_impl() {
        let mut canister_data = CanisterData::default();

        let all_token_transactions_from_individual_user_canister_chunk = vec![
            (
                0,
                TokenEventV1::Mint {
                    details: MintEvent::NewUserSignup {
                        new_user_principal_id: SPrincipal(get_mock_user_alice_principal_id()),
                    },
                    timestamp: SystemTime::now(),
                },
            ),
            (
                1,
                TokenEventV1::Mint {
                    details: MintEvent::Referral {
                        referee_user_principal_id: SPrincipal(get_mock_user_alice_principal_id()),
                        referrer_user_principal_id: SPrincipal(get_mock_user_bob_principal_id()),
                    },
                    timestamp: SystemTime::now(),
                },
            ),
        ];

        receive_all_token_transactions_from_individual_user_canister_impl(
            &mut canister_data,
            all_token_transactions_from_individual_user_canister_chunk.clone(),
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_none());

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_bob_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_all_token_transactions_from_individual_user_canister_impl(
            &mut canister_data,
            all_token_transactions_from_individual_user_canister_chunk.clone(),
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_some());
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .token_data
                .utility_token_transaction_history_v1
                .len(),
            0
        );

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_alice_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_all_token_transactions_from_individual_user_canister_impl(
            &mut canister_data,
            all_token_transactions_from_individual_user_canister_chunk,
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_some());
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .token_data
                .utility_token_transaction_history_v1
                .len(),
            2
        );
    }
}
