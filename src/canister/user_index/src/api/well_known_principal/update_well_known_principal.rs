use candid::Principal;
use ic_cdk::notify;
use ic_cdk_macros::update;
use shared_utils::common::{types::known_principal::KnownPrincipalType, utils::{permissions::is_caller_controller, task::run_task_concurrently}};

use crate::CANISTER_DATA;


#[update(guard="is_caller_controller")]
fn update_well_known_principal(known_principal_type: KnownPrincipalType, value: Principal) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.configuration.known_principal_ids.insert(known_principal_type, value);
    });

    issue_update_known_principal_for_individual_canisters(known_principal_type.clone(), value);
}


fn issue_update_known_principal_for_individual_canisters(known_principal_type: KnownPrincipalType, value: Principal) {
    let all_canisters = CANISTER_DATA.with_borrow(|canister_data| {
        let mut all_canisters: Vec<Principal> = canister_data.user_principal_id_to_canister_id_map.values().copied().collect();
        let mut available_canisters: Vec<Principal> = canister_data.available_canisters.iter().copied().collect();
        all_canisters.extend(available_canisters.drain(..));
        all_canisters
    }); 


    all_canisters.iter().for_each(|individual_canister_id| {
        let _ = notify(*individual_canister_id, "update_well_known_principal", (known_principal_type.clone(), value));
    });
}