use candid::{Nat, Principal};
use ic_cdk_macros::{query, update};
use icrc_ledger_types::{
    icrc1::account::Account,
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cents::CentsToken, pump_n_dump::{BalanceInfo, ParticipatedGameInfo, PumpNDumpStateDiff, PumpsAndDumps}, session::SessionType, token::TokenTransactions
    },
    common::{
        types::{
            known_principal::KnownPrincipalType,
            utility_token::token_event::{TokenEvent, WithdrawEvent},
        },
        utils::{
            permissions::{is_caller_controller, is_caller_global_admin_v2},
            system_time::get_current_system_time,
        },
    },
    pagination,
};

use crate::{data_model::pump_n_dump::NatStore, CANISTER_DATA, PUMP_N_DUMP};

async fn redeem_gdollr_inner(amount: u128, balance_fetch: impl FnOnce(&CentsToken) -> Nat) -> Result<(), String> {
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
        if balance_fetch(&pd.cents) < amount {
            return Err("Not enough balance".to_string());
        }

        pd.cents.handle_token_event(TokenEvent::Withdraw {
            amount,
            event_type: WithdrawEvent::WithdrawRequest,
        });

        Ok(())
    })?;

    let res = ic_cdk::call::<_, (Result<(), String>,)>(
        user_index,
        "redeem_gdollr",
        (profile_owner, amount),
    )
    .await;

    match res {
        Ok((Err(e),)) | Err((_, e)) => {
            PUMP_N_DUMP.with_borrow_mut(|pd| {
                pd.cents.handle_token_event(TokenEvent::Withdraw {
                    amount,
                    event_type: WithdrawEvent::WithdrawRequestFailed,
                })
            });
            Err(e)
        }
        Ok((Ok(()),)) => Ok(()),
    }
}

#[update]
pub async fn redeem_gdollr(amount: u128) -> Result<(), String> {
    redeem_gdollr_inner(amount, CentsToken::withdrawable_balance).await
}

#[update]
pub async fn redeem_gdolr_v2(amount: u128) -> Result<(), String> {
    redeem_gdollr_inner(amount, CentsToken::withdrawable_balance_v2).await
}

#[update]
pub fn reconcile_user_state(games: Vec<PumpNDumpStateDiff>) -> Result<(), String> {
    CANISTER_DATA.with_borrow(|cdata| is_caller_global_admin_v2(&cdata.known_principal_ids))?;

    PUMP_N_DUMP.with_borrow_mut(|pump_and_dump| {
        for game in games {
            let token_events = game.get_token_events_from_pump_dump_state_diff();

            for token_event in token_events {
                pump_and_dump.cents.handle_token_event(token_event);
            }

            if let PumpNDumpStateDiff::Participant(info) = game {
                pump_and_dump.games.push(info);
                pump_and_dump.total_dumps += info.dumps;
                pump_and_dump.total_pumps += info.pumps;
            }
        }

        Ok(())
    })
}

#[update]
pub async fn add_dollr_to_liquidity_pool(pool_root: Principal, amount: Nat) -> Result<(), String> {
    CANISTER_DATA.with_borrow(|cdata| is_caller_global_admin_v2(&cdata.known_principal_ids))?;

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
pub async fn stake_dollr_for_gdollr(amount: u128) -> Result<(), String> {
    let (ledger_id, user_index) = CANISTER_DATA
        .with_borrow(|cdata| {
            let ledger_id = cdata
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdSnsLedger)
                .copied()?;
            let user_index = cdata
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .copied()?;

            Some((ledger_id, user_index))
        })
        .ok_or("Unavailable")?;

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
            to: Account {
                owner: user_index,
                subaccount: None,
            },
            amount: Nat::from(amount),
            fee: None,
            memo: None,
            created_at_time: None,
        },),
    )
    .await
    .map_err(|(_, e)| e)?;

    res.0.map_err(|e| format!("{e:?}"))?;

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        pd.cents.handle_token_event(TokenEvent::Receive {
            amount: amount as u64,
            from_account: caller,
            timestamp: get_current_system_time(),
        });
    });

    Ok(())
}

#[update(guard = "is_caller_controller")]
pub fn update_pd_onboarding_reward(new_reward: Nat) -> Result<(), String> {
    PUMP_N_DUMP.with_borrow_mut(|pd| pd.onboarding_reward = new_reward);

    Ok(())
}

#[query]
fn pumps_and_dumps() -> PumpsAndDumps {
    PUMP_N_DUMP.with_borrow(|pd| pd.get_pumps_dumps())
}

#[query]
pub fn played_game_count() -> usize {
    PUMP_N_DUMP.with_borrow(|pd| pd.games.len())
}

#[query]
pub fn played_game_info_with_pagination_cursor(
    from_inclusive_index: u64,
    limit: u64,
) -> Result<Vec<ParticipatedGameInfo>, String> {
    PUMP_N_DUMP.with_borrow(|pd| {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            pd.games.len() as u64,
        )
        .map_err(|e| format!("{e:?}"))?;

        Ok(
            pd.games[from_inclusive_index as usize..(from_inclusive_index + limit) as usize]
                .to_vec(),
        )
    })
}

#[query]
pub fn pd_balance_info() -> BalanceInfo {
    PUMP_N_DUMP.with_borrow(|pd| BalanceInfo {
        net_airdrop_reward: pd.cents.get_net_airdrop(),
        balance: (pd.cents.get_current_token_balance()).into(),
        withdrawable: pd.cents.withdrawable_balance(),
    })
}

#[query]
pub fn pd_balance_info_v2() -> BalanceInfo {
    PUMP_N_DUMP.with_borrow(|pd| BalanceInfo {
        net_airdrop_reward: pd.cents.get_net_airdrop(),
        balance: (pd.cents.get_current_token_balance()).into(),
        withdrawable: pd.cents.withdrawable_balance_v2(),
    })
}

#[query]
pub fn net_earnings() -> Nat {
    PUMP_N_DUMP.with_borrow(|pd| pd.cents.get_net_earnings())
}
