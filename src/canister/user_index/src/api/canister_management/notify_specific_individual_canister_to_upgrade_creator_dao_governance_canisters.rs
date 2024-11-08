use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::{
    is_caller_controller, is_caller_controller_or_global_admin,
};

use crate::util::types::individual_user_canister::IndividualUserCanister;

#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn notify_specific_individual_canister_to_upgrade_creator_dao_governance_canisters(
    individual_canister_id: Principal,
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    let individual_canister = IndividualUserCanister::new(individual_canister_id)?;
    individual_canister.notify_to_upgrade_creator_dao_governance_canisters(wasm_module)
}
