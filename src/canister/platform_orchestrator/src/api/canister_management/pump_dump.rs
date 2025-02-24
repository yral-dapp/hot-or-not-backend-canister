use candid::Nat;
use futures::{stream::FuturesUnordered, StreamExt};
use ic_cdk::update;
use shared_utils::common::utils::permissions::is_caller_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_global_admin")]
pub fn update_pd_onboarding_reward_for_all_subnets(new_reward: Nat) -> Result<(), String> {
    let mut update_futs = CANISTER_DATA.with_borrow(|cdata| {
        let update_futs = cdata
            .all_subnet_orchestrator_canisters_list
            .iter()
            .map(|can| {
                ic_cdk::call::<_, ()>(
                    *can,
                    "update_pd_onboarding_reward_for_all_individual_users",
                    (new_reward.clone(),)
                )
            }).collect::<FuturesUnordered<_>>();

        Ok::<_, String>(update_futs)
    })?;

    ic_cdk::spawn(async move {
        while let Some(res) = update_futs.next().await {
            if let Err(e) = res {
                ic_cdk::eprintln!(
                    "failed to update pump dump onboarding reward for subnets. code: {:?}, err: {}",
                    e.0,
                    e.1
                )
            }
        }
    });

    Ok(())
}