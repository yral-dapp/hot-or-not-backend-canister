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

#[update]
pub async fn swap_request_action(token_pairs: TokenPairs, requester: Principal) -> Result<(), SwapError>{
    //auth
    if ic_cdk::caller() != get_profile_details_v2().principal_id{
        return Err(SwapError::Unauthenticated);
    }
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