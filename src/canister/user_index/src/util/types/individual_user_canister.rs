use candid::Principal;
use ic_cdk::api::management_canister::main::{canister_status, deposit_cycles, CanisterIdRecord};
use shared_utils::cycles::calculate_recharge_and_threshold_cycles_for_canister;

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
        let current_user_canister_balance =
            u128::try_from(user_canister_status.cycles.0).map_err(|e| e.to_string())?;

        let (threeshold, recharge_amount) =
            calculate_recharge_and_threshold_cycles_for_canister(idle_cycles_burned_in_a_day, None);

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
}
