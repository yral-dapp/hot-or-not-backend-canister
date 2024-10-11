use candid::Principal;
use ic_cdk::api::management_canister::main::{
    canister_status, deposit_cycles, update_settings, CanisterIdRecord, CanisterSettings,
    LogVisibility, UpdateSettingsArgument,
};
use ic_stable_structures::Log;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::UpgradeCanisterArg,
    cycles::calculate_threshold_and_recharge_cycles_for_canister,
};

use crate::CANISTER_DATA;

#[derive(Clone, Copy)]
pub struct IndividualUserCanister {
    pub canister_id: Principal,
}

impl IndividualUserCanister {
    pub fn new(canister_id: Principal) -> Result<Self, String> {
        let res = CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .user_principal_id_to_canister_id_map
                .iter()
                .find(move |(_, &user_canister)| user_canister == canister_id)
                .map(|(_, user_canister_id)| *user_canister_id)
        });

        if let Some(user_canister_id) = res {
            Ok(Self {
                canister_id: user_canister_id,
            })
        } else {
            Err(format!("Canister Id {canister_id} not found in the subnet"))
        }
    }

    pub async fn recharge_individual_canister(&self) -> Result<(), String> {
        let (user_canister_status,) = canister_status(CanisterIdRecord {
            canister_id: self.canister_id,
        })
        .await
        .map_err(|e| e.1)?;

        let idle_cycles_burned_in_a_day =
            u128::try_from(user_canister_status.idle_cycles_burned_per_day.0)
                .map_err(|e| e.to_string())?;
        let reserved_cycles =
            u128::try_from(user_canister_status.reserved_cycles.0).map_err(|e| e.to_string())?;
        let current_user_canister_balance =
            u128::try_from(user_canister_status.cycles.0).map_err(|e| e.to_string())?;

        let (threeshold, recharge_amount) = calculate_threshold_and_recharge_cycles_for_canister(
            idle_cycles_burned_in_a_day,
            reserved_cycles,
            None,
        );

        if current_user_canister_balance <= threeshold {
            return deposit_cycles(
                CanisterIdRecord {
                    canister_id: self.canister_id,
                },
                recharge_amount,
            )
            .await
            .map_err(|e| e.1);
        }

        Ok(())
    }

    pub async fn make_individual_canister_logs_public(&self) -> Result<(), String> {
        update_settings(UpdateSettingsArgument {
            canister_id: self.canister_id,
            settings: CanisterSettings {
                log_visibility: Some(LogVisibility::Public),
                ..Default::default()
            },
        })
        .await
        .map_err(|e| e.1)
    }

    pub async fn make_indvidual_canister_logs_private(&self) -> Result<(), String> {
        update_settings(UpdateSettingsArgument {
            canister_id: self.canister_id,
            settings: CanisterSettings {
                log_visibility: Some(LogVisibility::Controllers),
                ..Default::default()
            },
        })
        .await
        .map_err(|e| e.1)
    }

    pub fn allot_empty_canister(&self) -> Result<Principal, String> {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let Some(new_canister_id) = canister_data.backup_canister_pool.iter().next().copied()
            else {
                return Err("No Backup Canisters Available".into());
            };

            canister_data.backup_canister_pool.remove(&new_canister_id);

            Ok(new_canister_id)
        })
    }
}
