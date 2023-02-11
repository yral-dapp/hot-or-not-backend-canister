use candid::Principal;
use ic_cdk::api::call;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

pub async fn update_locally_stored_well_known_principals() {
    // TODO: enable this and remove the below
    // extract the canister ID of the configuration canister from well-known principals
    // let config_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
    //         canister_data_ref_cell
    //         .borrow()
    //         .known_principal_ids
    //         .get(&KnownPrincipalType::CanisterIdConfiguration)
    //         .unwrap()
    //         .clone()
    // });

    // TODO: Remove this once refactored
    let config_canister_id = match option_env!("DFX_NETWORK") {
        Some("ic") => Principal::from_text("efsfj-sqaaa-aaaap-qatwa-cai").unwrap(),
        _ => CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow()
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdConfiguration)
                .unwrap()
                .clone()
        }),
    };

    // fetch the well-known principals from the configuration canister
    let (well_known_principals,): (Vec<(KnownPrincipalType, Principal)>,) = call::call(
        config_canister_id,
        "get_current_list_of_all_well_known_principal_values",
        (),
    )
    .await
    .expect("Failed to fetch the well-known principals from the configuration canister");

    // update the locally stored well-known principals
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        canister_data.known_principal_ids = well_known_principals
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();
    });
}
