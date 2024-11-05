use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk_macros::update;
use ic_sns_root::pb::v1::{ListSnsCanistersRequest, ListSnsCanistersResponse};
use icrc_ledger_types::icrc1::{account::Account, transfer::{Memo, TransferArg, TransferError}};
use shared_utils::canister_specific::individual_user_template::types::{error::AirdropError, profile::UserProfileDetailsForFrontendV2};

use crate::CANISTER_DATA;

// TODO: Add checks to verify if the user_canister is from our network of yral canisters
#[update]
async fn request_airdrop(token_root: Principal, memo: Option<Memo>, amount: Nat, user_canister: Principal) -> Result<(), AirdropError> {
    let current_caller = ic_cdk::caller();
    let profile_info = get_profile_info(user_canister).await?;
    
    if profile_info.principal_id != current_caller {
        return Err(AirdropError::CanisterPrincipalDoNotMatch);
    }

    if !is_airdrop_unclaimed(token_root, &current_caller)? {// assertion for token owner is checked here will return err if deployed sns cans not found
        return Err(AirdropError::AlreadyClaimedAirdrop);
    }

    let decimals = get_decimals_from_token_root(token_root).await?;
    let amount = amount.min(Nat::from(1000u32) * 10u64.pow(decimals.into())); 
    if amount < 100u64.pow(decimals.into()) {
        return Err(AirdropError::RequestedAmountTooLow)
    }

    set_airdrop_claiming(token_root, current_caller);

    request_airdrop_internal(token_root, current_caller, memo, amount).await.inspect_err(|_|{
        CANISTER_DATA.with_borrow_mut(|cans_data| {
            cans_data
                .cdao_canisters
                .iter_mut()
                .find(|cdao| cdao.root == token_root)
                .map(|cdao| cdao.airdrop_info.set_airdrop_unclaimed(current_caller)).unwrap(); // can safely unwrap updating the states for the airdrop for the user creates it in place if not exists
        });
    })?; // rollback to unclaimed if error

    set_airdrop_claimed(token_root, current_caller); 
    Ok(())
}

async fn get_decimals_from_token_root(token_root: Principal) -> Result<u8, AirdropError>{
    let res: (ListSnsCanistersResponse,) = ic_cdk::call(token_root, "list_sns_canisters", (ListSnsCanistersRequest {},)).await?;

    let decimals: (u8,) = ic_cdk::call(res.0.ledger.ok_or(AirdropError::InvalidRoot).map(|l| l.into())?, "icrc1_decimals", ()).await?;
    Ok(decimals.0)
}

async fn request_airdrop_internal(token_root: Principal, current_caller: Principal, memo: Option<Memo>, amount: Nat) -> Result<(), AirdropError> {
    let ledger = get_ledger(token_root).await?;
    let balance = get_balance(ledger.into()).await?;
    
    if balance < amount {
        return Err(AirdropError::NoBalance);
    }

    transfer_tokens(ledger.into(), current_caller, memo, amount).await?;
    Ok(())
}
async fn get_profile_info(user_canister: Principal) -> Result<UserProfileDetailsForFrontendV2, AirdropError> {
    let (profile_info,): (UserProfileDetailsForFrontendV2,) = ic_cdk::call(user_canister, "get_profile_details_v2", ()).await?;
    Ok(profile_info)
}

fn is_airdrop_unclaimed(token_root: Principal, current_caller: &Principal) -> Result<bool, AirdropError> {
    CANISTER_DATA.with_borrow(|cans_data| {
        cans_data.cdao_canisters.iter().find(|cdao| cdao.root == token_root)
            .map(|cdao| cdao.airdrop_info.is_airdrop_unclaimed(current_caller))
    }).ok_or(AirdropError::InvalidRoot)
}

fn set_airdrop_claiming(token_root: Principal, current_caller: Principal) {
    CANISTER_DATA.with_borrow_mut(|cans_data| {
        if let Some(cdao) = cans_data.cdao_canisters.iter_mut().find(|cdao| cdao.root == token_root) {
            cdao.airdrop_info.set_airdrop_claiming(current_caller)
        }
    });
    ic_cdk::println!("Setting airdrop claiming for user: {:?}", current_caller);
}

async fn get_ledger(token_root: Principal) -> Result<PrincipalId, AirdropError> {
    let res: (ListSnsCanistersResponse,) = ic_cdk::call(token_root, "list_sns_canisters", (ListSnsCanistersRequest {},)).await?;
    res.0.ledger.ok_or(AirdropError::InvalidRoot)
}

async fn get_balance(ledger: Principal) -> Result<Nat, AirdropError> {
    let account = Account { owner: ic_cdk::id(), subaccount: None };
    let (balance_res,): (Nat,) = ic_cdk::call(ledger, "icrc1_balance_of", (account,)).await?;
    Ok(balance_res)
}

async fn transfer_tokens(ledger: Principal, current_caller: Principal, memo: Option<Memo>, amount: Nat) -> Result<(), AirdropError> {
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: current_caller, subaccount: None },
        fee: None,
        created_at_time: None,
        memo,
        amount,
    };
    let transfer_res: (Result<Nat, TransferError>,) = ic_cdk::call(ledger, "icrc1_transfer", (transfer_args,)).await?;
    let _ = transfer_res.0.map_err(AirdropError::Transfer)?;
    Ok(())
}

fn set_airdrop_claimed(token_root: Principal, current_caller: Principal) {
    CANISTER_DATA.with_borrow_mut(|cans_data| {
        if let Some(cdao) = cans_data.cdao_canisters.iter_mut().find(|cdao| cdao.root == token_root) {
            cdao.airdrop_info.set_airdrop_claimed(current_caller)
        }
    })
}

