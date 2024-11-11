use candid::{CandidType, Nat, Principal};
use ic_cdk::api::{management_canister::main::CanisterInfoRequest, time};
use ic_cdk_macros::{query, update};
use icrc_ledger_types::icrc2::{allowance::{Allowance, AllowanceArgs}, approve::{ApproveArgs, ApproveError}, transfer_from::{TransferFromArgs, TransferFromError}};
use serde::Deserialize;
use shared_utils::canister_specific::individual_user_template::types::{cdao::{DeployedCdaoCanisters, SwapRequestActions, TokenPairs}, error::SwapError};
use std::str::FromStr;

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
const XRC_FETCHABLE_TOKENS_LEDGER_IDS:[&str; 4] = ["xevnm-gaaaa-aaaar-qafnq-cai", "mxzaz-hqaaa-aaaar-qaada-cai", "ss2fx-dyaaa-aaaar-qacoq-cai", "ryjl3-tyaaa-aaaaa-aaaba-cai"]; // [ckusdc, ckbtc, cketh, icp]
#[derive(CandidType, Deserialize)]
pub enum AssetClass { Cryptocurrency, FiatCurrency }

#[derive(CandidType, Deserialize)]
pub struct Asset { pub class: AssetClass, pub symbol: String }

#[derive(CandidType, Deserialize)]
pub struct GetExchangeRateRequest {
  pub timestamp: Option<u64>,
  pub quote_asset: Asset,
  pub base_asset: Asset,
}

#[derive(CandidType, Deserialize)]
pub struct ExchangeRateMetadata {
  pub decimals: u32,
  pub forex_timestamp: Option<u64>,
  pub quote_asset_num_received_rates: u64,
  pub base_asset_num_received_rates: u64,
  pub base_asset_num_queried_sources: u64,
  pub standard_deviation: u64,
  pub quote_asset_num_queried_sources: u64,
}

#[derive(CandidType, Deserialize)]
pub struct ExchangeRate {
  pub metadata: ExchangeRateMetadata,
  pub rate: u64,
  pub timestamp: u64,
  pub quote_asset: Asset,
  pub base_asset: Asset,
}

#[derive(CandidType, Deserialize)]
pub enum ExchangeRateError {
  AnonymousPrincipalNotAllowed,
  CryptoQuoteAssetNotFound,
  FailedToAcceptCycles,
  ForexBaseAssetNotFound,
  CryptoBaseAssetNotFound,
  StablecoinRateTooFewRates,
  ForexAssetsNotFound,
  InconsistentRatesReceived,
  RateLimited,
  StablecoinRateZeroRate,
  Other{ code: u32, description: String },
  ForexInvalidTimestamp,
  NotEnoughCycles,
  ForexQuoteAssetNotFound,
  StablecoinRateNotFound,
  Pending,
}

#[derive(CandidType, Deserialize)]
pub enum GetExchangeRateResult { Ok(ExchangeRate), Err(ExchangeRateError) }

#[derive(CandidType, Deserialize)]
pub struct PairInfoExt {
  pub id: String,
  #[serde(rename = "price0CumulativeLast")]
  pub price0_cumulative_last: candid::Nat,

  pub creator: Principal,
  pub reserve0: candid::Nat,
  pub reserve1: candid::Nat,
  pub lptoken: String,

  #[serde(rename = "totalSupply")]
  pub total_supply: candid::Nat,
  pub token0: String,
  pub token1: String,

  #[serde(rename = "price1CumulativeLast")]
  pub price1_cumulative_last: candid::Nat,
  #[serde(rename = "kLast")]
  pub k_last: candid::Nat,

  #[serde(rename = "blockTimestampLast")]
  pub block_timestamp_last: candid::Int,
}


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
            let token_a_price = get_token_price(token_a.ledger).await;
            
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

async fn get_token_price(token_ledger: Principal) -> Option<f64> {
    if let Some(price) = get_token_price_from_creator_canister(token_ledger).await {
        Some(price)
    } else if XRC_FETCHABLE_TOKENS_LEDGER_IDS
        .iter()
        .any(|id| token_ledger.to_text() == *id)
    {
        get_token_price_from_xrc(token_ledger).await
    } else {
        let icpswap_price = get_token_price_from_icpswap(token_ledger).await;
        let sonicswap_price = get_token_price_from_sonicswap(token_ledger).await;

        if icpswap_price.is_some() && sonicswap_price.is_some() {
            Some((icpswap_price.unwrap() + sonicswap_price.unwrap()) / 2.0)
        } else {
            icpswap_price.or(sonicswap_price)
        }
    }
}

pub async fn get_token_price_from_icpswap(token_ledger: Principal) -> Option<f64>{
    let price: (PriceData, ) = ic_cdk::call(Principal::from_text("moe7a-tiaaa-aaaag-qclfq-cai").unwrap(), "getToken", (token_ledger.to_text(), )).await.ok()?;

    Some(price.0.price_usd)
}

pub async fn get_token_price_from_sonicswap(token_ledger: Principal) -> Option<f64> {
    let icp_ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").ok()?;

    let (token_icp_pair_option,): (Option<PairInfoExt>,) = ic_cdk::call(
        Principal::from_text("3xwpq-ziaaa-aaaah-qcn4a-cai").ok()?,
        "getPair",
        (token_ledger, icp_ledger),
    )
    .await
    .ok()?;

    let token_icp_pair = token_icp_pair_option?;

    let icp_price = get_token_price_from_xrc(icp_ledger).await?;

    let reserve0_f64 = nat_to_f64(&token_icp_pair.reserve0)?;
    let reserve1_f64 = nat_to_f64(&token_icp_pair.reserve1)?;

    if reserve0_f64 == 0.0 {
        return None;
    }

    let token_price_in_icp = reserve1_f64 / reserve0_f64;

    let token_price_in_usd = token_price_in_icp * icp_price;

    Some(token_price_in_usd)
}

fn nat_to_f64(n: &Nat) -> Option<f64> {
    let n_str = n.to_string();
    f64::from_str(&n_str).ok()
}

pub async fn get_token_price_from_xrc(token_ledger: Principal) -> Option<f64>{
    let symbol: (String, ) = ic_cdk::call(token_ledger, "icrc1_symbol", ()).await.ok()?;
    let symbol = symbol.0.replace("ck", "");
    let exchange_rate:(GetExchangeRateResult, ) = ic_cdk::call(Principal::from_text("uf6dk-hyaaa-aaaaq-qaaaq-cai").unwrap(), "get_exchange_rate", (GetExchangeRateRequest{
        base_asset:Asset{
            class: AssetClass::Cryptocurrency,
            symbol
        },
        quote_asset: Asset{
            class: AssetClass::FiatCurrency,
            symbol: "USD".to_string()
        },
        timestamp: None
    }, )).await.ok()?;

    match exchange_rate.0{
        GetExchangeRateResult::Ok(exchange_rate) => Some(exchange_rate.rate as f64),
        _ => None
    }
}

pub async fn get_token_price_from_creator_canister(token_ledger: Principal) -> Option<f64>{
    let owner_canister = get_token_owner_from_ledger(token_ledger).await.ok()?;
    let price: (Option<f64>, ) = ic_cdk::call(owner_canister, "get_token_price_by_ledger", (token_ledger, )).await.ok()?;
    price.0
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