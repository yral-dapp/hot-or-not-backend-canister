use candid::{Int, Nat, Principal};
use ic_cdk::{query, update};
use icrc_ledger_types::{icrc1::account::Account, icrc2::transfer_from::{TransferFromArgs, TransferFromError}};
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[update]
pub async fn redeem_gdollr(amount: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let (profile_owner, user_index) = CANISTER_DATA.with_borrow_mut(|cdata| {
        let admin = cdata.known_principal_ids[&KnownPrincipalType::UserIdGlobalSuperAdmin];
        if admin != caller {
            return Err("Unauthorized".to_string());
        }
        let principal_id = cdata.profile.principal_id.ok_or("Unavailable")?;

        if cdata.pump_n_dump.dollr_balance < amount {
            return Err("Not enough balance".to_string());
        }
        cdata.pump_n_dump.dollr_balance -= amount.clone();

        let user_index = cdata.known_principal_ids[&KnownPrincipalType::CanisterIdUserIndex];
        Ok((principal_id, user_index))
    })?;

    let res = ic_cdk::call::<_, (Result<(), String>,)>(
        user_index,
        "redeem_gdollr",
        (profile_owner, amount.clone())
    ).await;

    match res {
        Ok((Err(e),)) | Err((_, e)) => {
            CANISTER_DATA.with_borrow_mut(|cdata| {
                cdata.pump_n_dump.dollr_balance += amount
            });
            Err(e)
        },
        Ok((Ok(()),)) => Ok(()),
    }
}

#[update]
pub async fn settle_gdollr_balance(delta: Int) -> Result<(), String> {
    let caller = ic_cdk::caller();
    CANISTER_DATA.with_borrow_mut(|cdata| {
        let admin = cdata.known_principal_ids[&KnownPrincipalType::UserIdGlobalSuperAdmin];
        if admin != caller {
            return Err("Unauthorized".to_string())
        }
        if delta > 0 {
            cdata.pump_n_dump.dollr_balance.0 += delta.0.to_biguint().unwrap();
        } else {
            cdata.pump_n_dump.dollr_balance.0 -= (-delta.0).to_biguint().unwrap();
        }

        Ok(())
    })
}

#[update]
pub async fn add_dollr_to_liquidity_pool(pool_root: Principal, amount: Nat) -> Result<(), String> {
    let caller = ic_cdk::caller();
    CANISTER_DATA.with_borrow_mut(|cdata| {
        let admin = cdata.known_principal_ids[&KnownPrincipalType::UserIdGlobalSuperAdmin];
        if admin != caller {
            return Err("Unauthorized".to_string())
        }
        cdata.pump_n_dump.liquidity_pools.entry(pool_root)
            .and_modify(|bal| *bal += amount.clone())
            .or_insert(amount);

        Ok(())
    })
}

#[update]
pub async fn stake_dollr_for_gdollr(amount: Nat) -> Result<(), String> {
    let (ledger_id, user_index) = CANISTER_DATA.with_borrow(|cdata| {
        let ledger_id = cdata.known_principal_ids
            .get(&KnownPrincipalType::CanisterIdSnsLedger)
            .copied()?;
        let user_index = cdata.known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .copied()?;

        Some((ledger_id, user_index))
    }).ok_or("Unavailable")?;

    let caller = ic_cdk::caller();
    let res: (Result<Nat, TransferFromError>,) = ic_cdk::call(
        ledger_id,
        "icrc2_transfer_from",
        (TransferFromArgs {
            spender_subaccount: None,
            from: Account {
                owner: caller,
                subaccount: None,
            },
            to: Account { owner: user_index, subaccount: None, },
            amount: amount.clone(),
            fee: None,
            memo: None,
            created_at_time: None,
        },)
    ).await.map_err(|(_, e)| e)?;

    res.0.map_err(|e| format!("{e:?}"))?;

    CANISTER_DATA.with_borrow_mut(|cdata| {
        cdata.pump_n_dump.dollr_balance += amount;
    });

    Ok(())
}

#[query]
pub async fn gdollr_balance() -> Nat {
    CANISTER_DATA.with_borrow(|cdata| cdata.pump_n_dump.dollr_balance.clone())
}
