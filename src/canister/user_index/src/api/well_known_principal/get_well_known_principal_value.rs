use candid::Principal;
use shared_utils::common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType};

use crate::CANISTER_DATA;

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_well_known_principal_value(principal_type: KnownPrincipalType) -> Option<Principal> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let known_principal_ids = &canister_data_ref_cell.borrow().configuration.known_principal_ids;
        get_well_known_principal_value_impl(&principal_type, known_principal_ids)
    })
}

fn get_well_known_principal_value_impl(
    principal_type: &KnownPrincipalType,
    known_principal_ids: &KnownPrincipalMap,
) -> Option<Principal> {
    known_principal_ids.get(principal_type).copied()
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_canister_id_configuration, get_mock_canister_id_post_cache,
        get_mock_canister_id_user_index,
    };

    use crate::data_model::CanisterData;

    use super::*;

    #[test]
    fn test_get_well_known_principal_value_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.configuration.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdConfiguration,
            get_mock_canister_id_configuration(),
        );
        canister_data.configuration.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdPostCache,
            get_mock_canister_id_post_cache(),
        );
        canister_data.configuration.known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );

        assert_eq!(
            get_well_known_principal_value_impl(
                &KnownPrincipalType::CanisterIdConfiguration,
                &canister_data.configuration.known_principal_ids
            ),
            Some(get_mock_canister_id_configuration())
        );
        assert_eq!(
            get_well_known_principal_value_impl(
                &KnownPrincipalType::CanisterIdPostCache,
                &canister_data.configuration.known_principal_ids
            ),
            Some(get_mock_canister_id_post_cache())
        );
        assert_eq!(
            get_well_known_principal_value_impl(
                &KnownPrincipalType::CanisterIdUserIndex,
                &canister_data.configuration.known_principal_ids
            ),
            Some(get_mock_canister_id_user_index())
        );
    }
}
