use candid::{Nat, Principal};
use futures::{stream, StreamExt};
use ic_cdk::update;
use icrc_ledger_types::icrc1::{
    account::Account,
    transfer::{TransferArg, TransferError},
};

use crate::{data_model::get_sns_ledger, CANISTER_DATA};

#[update]
pub async fn redeem_gdollr(to_principal: Principal, amount: Nat) -> Result<(), String> {
    let ledger_id = get_sns_ledger().ok_or("Unavailable")?;

    let caller = ic_cdk::caller();
    CANISTER_DATA.with_borrow(|cdata| {
        // check if caller is authorized
        cdata
            .user_principal_id_to_canister_id_map
            .get(&to_principal)
            .ok_or_else(|| "Unauthorized".to_string())
            .and_then(|user_canister| {
                if *user_canister == caller {
                    Ok(())
                } else {
                    Err("Unauthorized".to_string())
                }
            })
    })?;

    ic_cdk::call::<_, (Result<Nat, TransferError>,)>(
        ledger_id,
        "icrc1_transfer",
        (TransferArg {
            from_subaccount: None,
            to: Account {
                owner: to_principal,
                subaccount: None,
            },
            fee: None,
            created_at_time: None,
            memo: None,
            amount: amount.clone(),
        },),
    )
    .await
    .map_err(|(_code, e)| e)?
    .0
    .map_err(|e| format!("transfer failed {e:?}"))?;

    Ok(())
}

// #[update(guard = "is_caller_controller")]
// pub fn update_pd_onboarding_reward_for_all_individual_users(
//     new_reward: u128,
// ) -> Result<(), String> {
//     let mut update_futs = CANISTER_DATA.with_borrow_mut(|cdata| {
//         cdata.pump_dump_onboarding_reward = new_reward;
//         let cans = cdata
//             .available_canisters
//             .clone()
//             .into_iter()
//             .map(move |can| {
//                 ic_cdk::call::<_, ()>(can, "update_pd_onboarding_reward", (new_reward,))
//             });
//         let stream = stream::iter(cans);
//         stream.buffer_unordered(10)
//     });

//     ic_cdk::spawn(async move {
//         while let Some(res) = update_futs.next().await {
//             if let Err(e) = res {
//                 ic_cdk::eprintln!(
//                     "failed to update_pd_onboarding_reward. code: {:?}, err: {}",
//                     e.0,
//                     e.1
//                 )
//             }
//         }
//     });

//     Ok(())
// }
