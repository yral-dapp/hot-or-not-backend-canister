use candid::{CandidType, Nat, Principal};
use ic_cdk::api::time;
use ic_cdk_macros::update;
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use serde::Deserialize;
use serde_json::Value;
use shared_utils::canister_specific::individual_user_template::types::error::SwapError;

#[update]
pub async fn swap_request(token_pairs: TokenPairs) -> Result<(), SwapError>{
    let TokenPairs{token_a, token_b} = token_pairs;

    if !is_icrc2_supported_token(token_a.ledger).await? || !is_icrc2_supported_token(token_b.ledger).await?{
        return Err(SwapError::UnsupportedToken);
    }

    let start_time = time();
    let allocation_res: (Result<Nat, ApproveError>, ) = ic_cdk::call(token_a.ledger, "icrc2_approve", (ApproveArgs{
        from_subaccount: None,
        spender: ic_cdk::id().into(),
        amount: token_a.amt,
        expected_allowance: None,
        memo: None,
        expires_at: Some(start_time + SWAP_REQUEST_EXPIRY),
        fee: None,
        created_at_time: Some(start_time)
    }, )).await?;

    allocation_res.0.map_err(SwapError::ApproveError)?;
    // TODO: Add the request to the push notification
    Ok(())
}

const SWAP_REQUEST_EXPIRY: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 1 wk

async fn is_icrc2_supported_token(token_ledger: Principal) -> Result<bool, SwapError>{
    let res: (Vec<SupportedStandards>, ) = ic_cdk::call(token_ledger, "icrc1_supported_standards", ()).await?;
    Ok(res.0.iter().any(|v| v.name == "ICRC2"))
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
struct SupportedStandards{
    name: String,
    url: String
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub struct SwapTokenData{
    pub ledger: Principal,
    pub amt: Nat
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub struct TokenPairs{
    pub token_a: SwapTokenData,
    pub token_b: SwapTokenData
}
