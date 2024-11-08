use candid::{CandidType, Nat, Principal};
use ic_cdk::api::{management_canister::main::CanisterInfoRequest, time};
use ic_cdk_macros::{query, update};
use icrc_ledger_types::icrc2::{allowance::{Allowance, AllowanceArgs}, approve::{ApproveArgs, ApproveError}, transfer_from::{TransferFromArgs, TransferFromError}};
use serde::Deserialize;
use shared_utils::canister_specific::individual_user_template::types::{cdao::{DeployedCdaoCanisters, SwapRequestActions, TokenPairs}, error::SwapError};

use crate::{api::profile::get_profile_details_v2::{self, get_profile_details_v2}, CANISTER_DATA};

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
struct SupportedStandards{
    name: String,
    url: String
}

#[derive(CandidType, Deserialize, PartialEq, Debug)]
pub struct PriceData{
    id: Nat,
    #[serde(rename = "volumeUSD1d")]
    volume_usd_1d: f64,
    #[serde(rename = "volumeUSD7d")]
    volume_usd_7d: f64,
    #[serde(rename = "totalVolumeUSD")]
    total_volume_usd: f64,
    name: String,
    #[serde(rename = "volumeUSD")]
    volume_usd: f64,
    #[serde(rename = "feesUSD")]
    fees_usd: f64,
    #[serde(rename = "priceUSDChange")]
    price_usd_change: f64,
    address: String,
    #[serde(rename = "txCount")]
    tx_count: u64,
    #[serde(rename = "priceUSD")]
    price_usd: f64,
    standard: String,
    symbol: String
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
            let token_a_price = get_token_price(token_a.ledger).await?;
            
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

            if let Some(price) = token_a_price{
                CANISTER_DATA.with_borrow_mut(|data| {
                    if let Some(cdao) = data.cdao_canisters.iter_mut().find(|cdao| cdao.ledger == token_b.ledger){
                        cdao.last_swapped_price = Some(price);
                    }
                });
            }
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

async fn get_token_price(token_ledger: Principal) -> Result<Option<f64>, SwapError>{
    // let price: Result<(PriceData, ), _> = ic_cdk::call(Principal::from_text("moe7a-tiaaa-aaaag-qclfq-cai").unwrap(), "getToken", (token_ledger.to_text(), )).await;

    // match price{
    //     Ok((price, )) => Ok(Some(price.price_usd)),
    //     Err(_) => {
    //         let owner_canister = get_token_owner_from_ledger(token_ledger).await?;
    //         let price: (Option<f64>, ) = ic_cdk::call(owner_canister, "get_token_price_by_ledger", (token_ledger, )).await?;
    //         Ok(price.0)
    //     }
    // }

    let owner_canister = get_token_owner_from_ledger(token_ledger).await?;
    let price: (Option<f64>, ) = ic_cdk::call(owner_canister, "get_token_price_by_ledger", (token_ledger, )).await?;
    Ok(price.0)
}

#[query]
pub fn get_token_price_by_ledger(token_ledger: Principal) -> Result<Option<f64>, SwapError>{
    CANISTER_DATA.with_borrow(|data| {
        data.cdao_canisters.iter().find(|cdao| cdao.ledger == token_ledger).map(|cdao| cdao.last_swapped_price).ok_or(SwapError::NoController)
    })
}

async fn get_token_owner_from_ledger(ledger: Principal) -> Result<Principal, SwapError>{
    let res  = ic_cdk::api::management_canister::main::canister_info(CanisterInfoRequest{
        canister_id: ledger,
        num_requested_changes: None
    }).await?.0;
    for f in res.controllers.into_iter().filter(|f| f.to_text().ends_with("-cai")){
        let res: Result<(Vec<DeployedCdaoCanisters>, ), _> = ic_cdk::call(f, "deployed_cdao_canisters", ()).await;

        match res{
            Ok((cdao, )) => {
                if cdao.iter().any(|cdao| cdao.ledger == ledger){
                    return Ok(f);
                }
            },
            Err(_) => continue
        };
    }

    Err(SwapError::NoController)
}

async fn is_icrc2_supported_token(token_ledger: Principal) -> Result<bool, SwapError>{
    let res: (Vec<SupportedStandards>, ) = ic_cdk::call(token_ledger, "icrc1_supported_standards", ()).await?;
    Ok(res.0.iter().any(|v| v.name == "ICRC2"))
}