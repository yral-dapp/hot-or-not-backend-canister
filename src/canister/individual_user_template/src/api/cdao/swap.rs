use candid::{CandidType, Nat, Principal};
use ic_cdk::api::time;
use ic_cdk_macros::update;
use icrc_ledger_types::icrc2::{allowance::{Allowance, AllowanceArgs}, approve::{ApproveArgs, ApproveError}, transfer_from::{TransferFromArgs, TransferFromError}};
use serde::Deserialize;
use shared_utils::{canister_specific::individual_user_template::types::{cdao::{SwapRequestActions, TokenPairs}, error::SwapError}, common::utils::permissions::is_caller_global_admin};

use crate::{api::profile::get_profile_details_v2::get_profile_details_v2, CANISTER_DATA};

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
struct SupportedStandards{
    name: String,
    url: String
}

const SWAP_REQUEST_EXPIRY: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 1 wk


#[update]
pub async fn swap_request(token_pairs: TokenPairs) -> Result<(), SwapError>{
    let TokenPairs{token_a, token_b} = token_pairs;

    if !is_icrc2_supported_token(token_a.ledger).await?{
        return Err(SwapError::UnsupportedToken);
    }

    if CANISTER_DATA.with_borrow(|data| data.cdao_canisters.iter().find(|cdao| cdao.ledger == token_b.ledger).cloned()).is_none(){
        return Err(SwapError::IsNotTokenCreator);
    }

    
    // todo: replace this by a frontend server function instead because this uses the caller to infer the from field in approve...
    let previous_approval_amt = get_previous_approval_amount(ic_cdk::caller(), ic_cdk::id(), token_a.ledger).await?;
    let allocation_res: (Result<Nat, ApproveError>, ) = ic_cdk::call(token_a.ledger, "icrc2_approve", (ApproveArgs{
        from_subaccount: None,
        spender: ic_cdk::id().into(),
        amount: previous_approval_amt + token_a.amt,
        expected_allowance: None,
        memo: None,
        expires_at: Some(time() + SWAP_REQUEST_EXPIRY),
        fee: None,
        created_at_time: None
    }, )).await?;
    // ....


    allocation_res.0.map_err(SwapError::ApproveError)?;
    // TODO: Push notifications
    Ok(())
}

// Creator principal is the caller here
#[update]
pub async fn swap_request_action(op: SwapRequestActions) -> Result<(), SwapError>{
    //auth
    if ic_cdk::caller() != get_profile_details_v2().principal_id{
        return Err(SwapError::Unauthenticated);
    }
    match op{
        SwapRequestActions::Accept { token_pairs, requester } => {
            let TokenPairs{token_a, token_b} = token_pairs;

            let transfer_res: (Result<Nat, TransferFromError>, ) = ic_cdk::call(token_a.ledger, "icrc2_transfer_from", (TransferFromArgs{
                from: requester.into(),
                spender_subaccount: None,
                to: ic_cdk::caller().into(),
                amount: token_a.amt,
                fee: None,
                memo: None,
                created_at_time: None
            }, )).await?;
            transfer_res.0.map_err(SwapError::TransferFromError)?;

            let transfer_res: (Result<Nat, TransferFromError>, ) = ic_cdk::call(token_b.ledger, "icrc2_transfer_from", (TransferFromArgs{
                from: ic_cdk::caller().into(),
                spender_subaccount: None,
                to: requester.into(),
                amount: token_b.amt,
                fee: None,
                memo: None,
                created_at_time: None
            }, )).await?;
            transfer_res.0.map_err(SwapError::TransferFromError)?;
            
            // send push notifs to both parties
            // Cannot remove the approval as it is not possible to do so
        },
        SwapRequestActions::Reject { token_pairs, requester } => {
            // Reject and send push notifs to both parties
            // Cannot remove the approval as it is not possible to do so
        }
    }

    Ok(())
}

#[update(guard = "is_caller_global_admin")]
fn update_last_swap_price(token_ledger: Principal, price: f64){
    CANISTER_DATA.with_borrow_mut(|data| {
        if let Some(cdao) = data.cdao_canisters.iter_mut().find(|cdao| cdao.ledger == token_ledger){
            cdao.last_swapped_price = Some(price);
        }
    });
}

#[update]
pub async fn cancel_swap_request(token_pairs: TokenPairs) -> Result<(), SwapError>{
    let TokenPairs{token_a, ..} = token_pairs;
    let previous_approval_amt = get_previous_approval_amount(ic_cdk::caller(), ic_cdk::id(), token_a.ledger).await?;

    let allocation_res: (Result<Nat, ApproveError>, ) = ic_cdk::call(token_a.ledger, "icrc2_approve", (ApproveArgs{
        from_subaccount: None,
        spender: ic_cdk::id().into(),
        amount: previous_approval_amt - token_a.amt, // https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-2/README.md#alice-removes-her-allowance-for-canister-c
        expected_allowance: None,
        memo: None,
        expires_at: Some(time() + SWAP_REQUEST_EXPIRY),
        fee: None,
        created_at_time: None
    }, )).await?;

    allocation_res.0.map_err(SwapError::ApproveError)?;
    Ok(())
}

async fn get_previous_approval_amount(requester: Principal, spender: Principal, ledger: Principal) -> Result<Nat, SwapError>{
    let previous_approval: (Allowance, ) = ic_cdk::call(ledger, "icrc2_allowance", (AllowanceArgs{
        account: requester.into(),
        spender: spender.into()
    }, )).await?;

    Ok(previous_approval.0.allowance)
}

async fn is_icrc2_supported_token(token_ledger: Principal) -> Result<bool, SwapError>{
    let res: (Vec<SupportedStandards>, ) = ic_cdk::call(token_ledger, "icrc1_supported_standards", ()).await?;
    Ok(res.0.iter().any(|v| v.name == "ICRC-2"))
}