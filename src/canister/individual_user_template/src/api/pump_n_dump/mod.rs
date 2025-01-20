use candid::{Nat, Principal};
use ic_cdk::{query, update};
use icrc_ledger_types::{icrc1::account::Account, icrc2::transfer_from::{TransferFromArgs, TransferFromError}};
use shared_utils::{canister_specific::individual_user_template::types::pump_n_dump::{ParticipatedGameInfo, PumpNDumpStateDiff, PumpsAndDumps}, common::types::known_principal::KnownPrincipalType, pagination};

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

        if cdata.pump_n_dump.withdrawable_balance < amount {
            return Err("Not enough balance".to_string());
        }
        cdata.pump_n_dump.withdrawable_balance -= amount.clone();

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
                cdata.pump_n_dump.withdrawable_balance += amount
            });
            Err(e)
        },
        Ok((Ok(()),)) => Ok(()),
    }
}

#[update]
pub fn reconcile_user_state(games: Vec<PumpNDumpStateDiff>) -> Result<(), String> {
    let caller = ic_cdk::caller();
    CANISTER_DATA.with_borrow_mut(|cdata| {
        let admin = cdata.known_principal_ids[&KnownPrincipalType::UserIdGlobalSuperAdmin];
        if admin != caller {
            return Err("Unauthorized".to_string())
        }
        let mut to_deduct: Nat = 0u32.into();
        let mut to_add: Nat = 0u32.into();
        for game in games {
            match game {
                PumpNDumpStateDiff::Participant(info) => {
                    to_deduct += info.pumps + info.dumps;
                    to_add += info.reward.clone();
                    cdata.pump_n_dump.total_dumps += info.dumps;
                    cdata.pump_n_dump.total_pumps += info.pumps;
                    cdata.pump_n_dump.games.push(info);
                },
                PumpNDumpStateDiff::CreatorReward(reward) => {
                    to_add += reward;
                }
            }
        }
        let withdrawable_bal = &mut cdata.pump_n_dump.withdrawable_balance;
        *withdrawable_bal += to_add.clone();

        let game_only_bal = &mut cdata.pump_n_dump.game_only_balance;
        if &to_deduct <= game_only_bal {
            *game_only_bal -= to_deduct;
        } else {
            let deduct_from_withdrawable = to_deduct - game_only_bal.clone();
            *game_only_bal = 0u32.into();
            assert!(&deduct_from_withdrawable <= withdrawable_bal);
            *withdrawable_bal -= deduct_from_withdrawable;
        }

        cdata.pump_n_dump.net_earnings += to_add;

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
        cdata.pump_n_dump.withdrawable_balance += amount;
    });

    Ok(())
}

#[query]
pub fn pumps_and_dumps() -> PumpsAndDumps {
    CANISTER_DATA.with_borrow(|cdata| {
        PumpsAndDumps {
            pumps: cdata.pump_n_dump.total_pumps.clone(),
            dumps: cdata.pump_n_dump.total_dumps.clone(),
        }
    })
} 

#[query]
pub fn played_game_count() -> usize {
    CANISTER_DATA.with_borrow(|cdata| cdata.pump_n_dump.games.len())
}

#[query]
pub fn played_game_info_with_pagination_cursor(from_inclusive_index: u64, limit: u64) -> Result<Vec<ParticipatedGameInfo>, String> {
    CANISTER_DATA.with_borrow(|cdata| {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            cdata.pump_n_dump.games.len() as u64,
        ).map_err(|e| format!("{e:?}"))?;

        Ok(cdata.pump_n_dump.games[from_inclusive_index as usize..(from_inclusive_index+limit) as usize].to_vec())
    })
}

#[query]
pub fn gdollr_balance() -> Nat {
    CANISTER_DATA.with_borrow(|cdata| cdata.pump_n_dump.playable_balance())
}

#[query]
pub fn withdrawable_balance() -> Nat {
    CANISTER_DATA.with_borrow(|cdata| cdata.pump_n_dump.withdrawable_balance.clone())
}

#[query]
pub fn net_earnings() -> Nat {
    CANISTER_DATA.with_borrow(|cdata| cdata.pump_n_dump.net_earnings.clone())
}
