use candid::Nat;
use futures::{stream, StreamExt};
use ic_cdk::update;

use crate::{CANISTER_DATA, guard::is_caller::is_caller_global_admin_or_controller};

#[update(guard = "is_caller_global_admin_or_controller")]
pub fn update_pd_onboarding_reward_for_all_subnets(new_reward: Nat) -> Result<(), String> {
    let mut update_futs = CANISTER_DATA.with_borrow(|cdata| {
        let update_futs = cdata
            .all_subnet_orchestrator_canisters_list
            .clone()
            .into_iter()
            .map(move |can| {
                ic_cdk::call::<_, ()>(
                    can,
                    "update_pd_onboarding_reward_for_all_individual_users",
                    (new_reward.clone(),)
                )
            });
        let stream = stream::iter(update_futs);

        Ok::<_, String>(stream.buffer_unordered(10))
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