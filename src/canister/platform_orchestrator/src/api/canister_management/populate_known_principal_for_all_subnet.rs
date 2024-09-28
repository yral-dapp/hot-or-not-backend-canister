use candid::Principal;
use ic_cdk::call;
use ic_cdk_macros::update;
use shared_utils::common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType};

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
async fn populate_known_principal_for_all_subnet() {
    let subnet_orchestrators: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .iter()
            .copied()
            .collect()
    });

    for subnet_id in subnet_orchestrators {
        let (subnet_known_principals,): (Vec<(KnownPrincipalType, Principal)>,) = call(
            subnet_id,
            "get_current_list_of_all_well_known_principal_values",
            (),
        )
        .await
        .unwrap();
        let subnet_known_principal_map: KnownPrincipalMap =
            subnet_known_principals.into_iter().collect();
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .known_principals
                .subnet_orchestrator_known_principals_map
                .insert(subnet_id, subnet_known_principal_map)
        });
    }
}
