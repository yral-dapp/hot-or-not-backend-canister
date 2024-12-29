use candid::{Nat, Principal};
use ic_cdk::update;
use icrc_ledger_types::icrc1::{account::Account, transfer::{TransferArg, TransferError}};

use crate::{data_model::get_sns_ledger, CANISTER_DATA};

#[update]
pub async fn redeem_gdollr(to_principal: Principal, amount: Nat) -> Result<(), String> {
    let ledger_id = get_sns_ledger().ok_or("Unavailable")?;

    let caller = ic_cdk::caller();
    CANISTER_DATA.with_borrow(|cdata| {
        // check if caller is authorized
        cdata
            .user_principal_id_to_canister_id_map.get(&to_principal)
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
        },)
    ).await
    .map_err(|(_code, e)| e)?
    .0
    .map_err(|e| format!("transfer failed {e:?}"))?;

    Ok(())
}
