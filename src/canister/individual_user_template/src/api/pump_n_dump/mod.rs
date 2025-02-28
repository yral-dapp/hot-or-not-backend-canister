use candid::{Nat, Principal};
use ic_cdk::{query, update};
use icrc_ledger_types::{icrc1::account::Account, icrc2::transfer_from::{TransferFromArgs, TransferFromError}};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        pump_n_dump::{BalanceInfo, ParticipatedGameInfo, PumpNDumpStateDiff, PumpsAndDumps},
        session::SessionType
    },
    common::{
        types::known_principal::KnownPrincipalType,
        utils::permissions::{is_caller_global_admin_v2, is_caller_controller},
    },
    constant::GDOLLR_TO_E8S, pagination
};

use crate::{data_model::pump_n_dump::NatStore, CANISTER_DATA, PUMP_N_DUMP};

#[update]
pub async fn redeem_gdollr(amount: Nat) -> Result<(), String> {
    let (profile_owner, user_index) = CANISTER_DATA.with_borrow(|cdata| {
        if cdata.session_type != Some(SessionType::RegisteredSession) {
            return Err("Login required".to_string());
        }
        is_caller_global_admin_v2(&cdata.known_principal_ids)?;

        let principal_id = cdata.profile.principal_id.ok_or("Unavailable")?;

        let user_index = cdata.known_principal_ids[&KnownPrincipalType::CanisterIdUserIndex];
        Ok((principal_id, user_index))
    })?;

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        if pd.withdrawable_balance() < amount {
            return Err("Not enough balance".to_string());
        }
        pd.balance -= amount.clone();
        Ok(())
    })?;

    let res = ic_cdk::call::<_, (Result<(), String>,)>(
        user_index,
        "redeem_gdollr",
        (profile_owner, amount.clone())
    ).await;

    match res {
        Ok((Err(e),)) | Err((_, e)) => {
            PUMP_N_DUMP.with_borrow_mut(|pd| {
                pd.balance += amount
            });
            Err(e)
        },
        Ok((Ok(()),)) => Ok(()),
    }
}

#[update]
pub fn reconcile_user_state(games: Vec<PumpNDumpStateDiff>) -> Result<(), String> {
    CANISTER_DATA.with_borrow(|cdata| {
        is_caller_global_admin_v2(&cdata.known_principal_ids)
    })?;

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        let mut to_deduct: Nat = 0u32.into();
        let mut to_add: Nat = 0u32.into();
        for game in games {
            match game {
                PumpNDumpStateDiff::Participant(info) => {
                    to_deduct += info.pumps + info.dumps;
                    to_add += info.reward.clone();
                    pd.total_dumps += info.dumps;
                    pd.total_pumps += info.pumps;
                    pd.games.push(info);
                },
                PumpNDumpStateDiff::CreatorReward(reward) => {
                    to_add += reward;
                }
            }
        }
        to_deduct *= GDOLLR_TO_E8S;
        pd.balance += to_add.clone();
        pd.balance -= to_deduct;

        pd.net_earnings += to_add;

        Ok(())
    })
}

#[update]
pub async fn add_dollr_to_liquidity_pool(pool_root: Principal, amount: Nat) -> Result<(), String> {
    CANISTER_DATA.with_borrow(|cdata| {
        is_caller_global_admin_v2(&cdata.known_principal_ids)
    })?;

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        let lp = &mut pd.liquidity_pools;
        let to_insert = if let Some(mut cur_bal) = lp.get(&pool_root) {
            cur_bal.0 += amount;
            cur_bal
        } else {
            NatStore(amount)
        };
        lp.insert(pool_root, to_insert);

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

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        pd.balance += amount;
    });

    Ok(())
}

#[update(guard = "is_caller_controller")]
pub fn update_pd_onboarding_reward(new_reward: Nat) -> Result<(), String> {
    PUMP_N_DUMP.with_borrow_mut(|pd| pd.onboarding_reward = new_reward);

    Ok(())
}

#[query]
pub fn pumps_and_dumps() -> PumpsAndDumps {
    PUMP_N_DUMP.with_borrow(|pd| {
        PumpsAndDumps {
            pumps: pd.total_pumps.clone(),
            dumps: pd.total_dumps.clone(),
        }
    })
} 

#[query]
pub fn played_game_count() -> usize {
    PUMP_N_DUMP.with_borrow(|pd| pd.games.len())
}

#[query]
pub fn played_game_info_with_pagination_cursor(from_inclusive_index: u64, limit: u64) -> Result<Vec<ParticipatedGameInfo>, String> {
    PUMP_N_DUMP.with_borrow(|pd| {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            pd.games.len() as u64,
        ).map_err(|e| format!("{e:?}"))?;

        Ok(pd.games[from_inclusive_index as usize..(from_inclusive_index+limit) as usize].to_vec())
    })
}

#[query]
pub fn pd_balance_info() -> BalanceInfo {
    PUMP_N_DUMP.with_borrow(|pd| BalanceInfo {
        net_airdrop_reward: pd.net_airdrop.clone(),
        balance: pd.balance.clone(),
        withdrawable: pd.withdrawable_balance(),
    })
}

#[query]
pub fn net_earnings() -> Nat {
    PUMP_N_DUMP.with_borrow(|pd| pd.net_earnings.clone())
}
