use candid::Principal;
use ic_cdk::api::management_canister::main::{
    update_settings, CanisterSettings, UpdateSettingsArgument,
};
use ic_cdk_macros::update;
use shared_utils::common::{
    types::known_principal::KnownPrincipalType, utils::permissions::is_caller_controller,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
pub async fn set_controller_as_subnet_orchestrator(canister_id: Principal) {
    let subnet_orchestrator_canister_id = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .copied()
            .unwrap()
    });

    update_settings(UpdateSettingsArgument {
        canister_id,
        settings: CanisterSettings {
            controllers: Some(vec![subnet_orchestrator_canister_id]),
            ..Default::default()
        },
    })
    .await
    .unwrap();
}
