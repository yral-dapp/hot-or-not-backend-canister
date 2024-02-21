use std::collections::BTreeMap;

use candid::Principal;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[query]
fn get_user_canister_id_from_user_principal_id(user_id: Principal) -> Option<Principal> {
    if user_id == Principal::anonymous() {
        return None;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_user_canister_id_from_user_principal_id_impl(
            user_id,
            &canister_data_ref_cell
                .borrow()
                .user_principal_id_to_canister_id_map,
        )
    })
}

fn get_user_canister_id_from_user_principal_id_impl(
    user_id: Principal,
    user_principal_id_to_canister_id_map: &BTreeMap<Principal, Principal>,
) -> Option<Principal> {
    user_principal_id_to_canister_id_map.get(&user_id).cloned()
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
    };

    use crate::data_model::CanisterData;

    use super::*;

    #[test]
    fn test_get_user_canister_id_from_user_principal_id_impl() {
        let mut canister_data = CanisterData::default();

        assert_eq!(
            get_user_canister_id_from_user_principal_id_impl(
                get_mock_user_alice_principal_id(),
                &canister_data.user_principal_id_to_canister_id_map
            ),
            None
        );

        canister_data.user_principal_id_to_canister_id_map.insert(
            get_mock_user_alice_principal_id(),
            get_mock_user_alice_canister_id(),
        );

        assert_eq!(
            get_user_canister_id_from_user_principal_id_impl(
                get_mock_user_alice_principal_id(),
                &canister_data.user_principal_id_to_canister_id_map
            ),
            Some(get_mock_user_alice_canister_id())
        );
    }
}
