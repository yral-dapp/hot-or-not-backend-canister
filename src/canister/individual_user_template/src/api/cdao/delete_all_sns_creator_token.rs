use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{
        deposit_cycles, update_settings, CanisterIdRecord, CanisterSettings, UpdateSettingsArgument,
    },
    id,
};
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters, session::SessionType,
    },
    common::utils::permissions::is_caller_controller,
    constant::GLOBAL_SUPER_ADMIN_USER_ID,
    types::sns_canisters::swap::{self, GetLifecycleArg, Swap},
};

use crate::CANISTER_DATA;

use super::{
    request_cycles_from_subnet_orchestrator,
    utils::uninstall_code_and_return_empty_canisters_to_subnet_backup_pool,
};

#[update(guard = "is_caller_controller")]
pub fn delete_all_creator_token() {
    let deployed_canisters =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.cdao_canisters.clone());

    deployed_canisters
        .into_iter()
        .for_each(|deployed_cdao_canisters| {
            update_or_delete_sns_creator_token(deployed_cdao_canisters);
        });
}

pub fn update_or_delete_sns_creator_token(deployed_canisters: DeployedCdaoCanisters) {
    let canister_ids = deployed_canisters.get_canister_ids();
    let anonymous_session = CANISTER_DATA
        .with_borrow(|canister_data| {
            canister_data
                .session_type
                .map(|session| session == SessionType::AnonymousSession)
        })
        .unwrap_or(false);

    const UNINSTALL_RECHARGE_AMOUNT: u128 = 300_000_000_000;

    ic_cdk::spawn(async move {
        let _ = request_cycles_from_subnet_orchestrator(5 * UNINSTALL_RECHARGE_AMOUNT).await;
        for canister_id in canister_ids.iter() {
            let _ = deposit_cycles(
                CanisterIdRecord {
                    canister_id: *canister_id,
                },
                UNINSTALL_RECHARGE_AMOUNT,
            )
            .await;
        }

        let needs_to_be_deleted =
            anonymous_session || is_sns_canister_in_invalid_state(deployed_canisters.swap).await;

        if needs_to_be_deleted {
            uninstall_code_and_return_empty_canisters_to_subnet_backup_pool(canister_ids).await;
            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data
                    .cdao_canisters
                    .retain(|key| *key != deployed_canisters);
            });

            ic_cdk::println!(
                "Deleting creator SNS canisters with Root Canister Id {}",
                deployed_canisters.root.to_text()
            )
        } else {
            update_controllers_for_ledger_and_swap(deployed_canisters).await
        }
    });
}

pub async fn is_sns_canister_in_invalid_state(swap_canister_id: Principal) -> bool {
    let swap_canister = swap::Service(swap_canister_id);

    let swap_lifecycle_res = swap_canister.get_lifecycle(GetLifecycleArg {}).await;

    match swap_lifecycle_res {
        Ok((lifecycle,)) => {
            if let Some(lifecycle) = lifecycle.lifecycle {
                lifecycle == 4
            } else {
                false
            }
        }
        Err(_e) => false,
    }
}

pub async fn update_controllers_for_ledger_and_swap(deployed_canister: DeployedCdaoCanisters) {
    let global_super_admin_principal = Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap();

    let _ = update_settings(UpdateSettingsArgument {
        canister_id: deployed_canister.ledger,
        settings: CanisterSettings {
            controllers: Some(vec![
                global_super_admin_principal,
                id(),
                deployed_canister.root,
            ]),
            ..Default::default()
        },
    })
    .await;

    let _ = update_settings(UpdateSettingsArgument {
        canister_id: deployed_canister.swap,
        settings: CanisterSettings {
            controllers: Some(vec![
                global_super_admin_principal,
                id(),
                deployed_canister.root,
                ic_nns_constants::ROOT_CANISTER_ID.into(),
            ]),
            ..Default::default()
        },
    })
    .await;
}
