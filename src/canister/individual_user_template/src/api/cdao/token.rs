use candid::{Nat, Principal};
use ic_cdk::{query, update};
use ic_sns_root::pb::v1::{ListSnsCanistersRequest, ListSnsCanistersResponse};
use icrc_ledger_types::icrc1::{account::Account, transfer::{Memo, TransferArg, TransferError}};
use shared_utils::{canister_specific::individual_user_template::types::error::CdaoTokenError, pagination::{self, PaginationError}};

use crate::CANISTER_DATA;

/// Add a new token
/// returns true if new token is added
#[update]
async fn add_token(root_canister: Principal) -> Result<bool, CdaoTokenError> {
    let token_added = CANISTER_DATA.with(|cdata| {
        let cdata = cdata.borrow();
        cdata.token_roots.contains_key(&root_canister)
    });
    if token_added {
        return Ok(false);
    }

    let res: (ListSnsCanistersResponse,) = ic_cdk::call(root_canister, "list_sns_canisters", (ListSnsCanistersRequest {},)).await?;
    let cans = res.0;
    let ledger = cans.ledger.ok_or(CdaoTokenError::InvalidRoot)?;

    let acc = Account { owner: ic_cdk::id(), subaccount: None };
    let balance: (Nat,) = ic_cdk::call(ledger.into(), "icrc1_balance_of", (acc,)).await?;
    if balance.0 == 0u32 {
        return Err(CdaoTokenError::NoBalance);
    }

    CANISTER_DATA.with(|cdata| {
        let mut cdata = cdata.borrow_mut();
        cdata.token_roots.insert(root_canister, ());
    });


    return Ok(true);
}

#[update]
async fn transfer_token_to_user_canister(token_root: Principal, target_canister: Principal, memo: Option<Memo>, amount: Nat) -> Result<(), CdaoTokenError> {
    let res: (ListSnsCanistersResponse,) = ic_cdk::call(token_root, "list_sns_canisters", (ListSnsCanistersRequest {},)).await?;
    let ledger = res.0.ledger.ok_or(CdaoTokenError::InvalidRoot)?;

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: target_canister, subaccount: None },
        fee: None,
        created_at_time: None,
        memo,
        amount,
    };
    let transfer_res: (Result<Nat, TransferError>,) = ic_cdk::call(ledger.into(), "icrc1_transfer", (transfer_args,)).await?;
    transfer_res.0.map_err(CdaoTokenError::Transfer)?;

    let res: (Result<bool, CdaoTokenError>,) = ic_cdk::call(target_canister, "add_token", (token_root,)).await?;
    // Rollback transfer on failure
    res.0.unwrap();

    Ok(())
}

#[query]
fn get_token_roots_of_this_user_with_pagination_cursor(from_inclusive_index: u64, limit: u64) -> Result<Vec<Principal>, PaginationError> {
    CANISTER_DATA.with_borrow(|cdata| {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            cdata.token_roots.len(),
        )?;
        let tokens = cdata.token_roots
            .iter()
            .skip(from_inclusive_index as usize)
            .take(limit as usize)
            .map(|(token, _)| token)
            .collect();
        Ok(tokens)
    })
}
