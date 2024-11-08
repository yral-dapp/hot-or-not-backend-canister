use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
pub fn notify_all_individual_canisters_to_upgrade_creator_dao_governance_canisters(
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let individual_canisters = canister_data.user_principal_id_to_canister_id_map.iter();

        for (_, canister_id) in individual_canisters {
            ic_cdk::notify::<_>(
                *canister_id,
                "upgrade_creator_dao_governance_canisters",
                (wasm_module.clone(),),
            )
            .map_err(|e| format!("Error: {:?}", e))?;
        }

        Ok(())
    })
}
