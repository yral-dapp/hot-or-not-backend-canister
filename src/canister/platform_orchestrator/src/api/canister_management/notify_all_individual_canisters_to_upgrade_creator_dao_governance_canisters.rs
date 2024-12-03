use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
pub fn notify_all_individual_canisters_to_upgrade_creator_dao_governance_canisters(
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let subnet_orchestrators = canister_data.subnet_orchestrators().iter();

        for canister_id in subnet_orchestrators {
            ic_cdk::notify::<_>(
                *canister_id,
                "notify_all_individual_canisters_to_upgrade_creator_dao_governance_canisters",
                (wasm_module.clone(),),
            )
            .map_err(|e| format!("Error: {:?}", e))?;
        }

        Ok(())
    })
}
