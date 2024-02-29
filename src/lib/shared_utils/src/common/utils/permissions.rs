use ic_cdk::{api::is_controller, caller};
use candid::Principal;
use crate::constant::RECLAIM_CANISTER_PRINCIPAL_ID;

pub fn is_reclaim_canister_id() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let reclaim_canister_principal = Principal::from_text(RECLAIM_CANISTER_PRINCIPAL_ID).unwrap();

    // Here accessing the args ???

    if caller == reclaim_canister_principal {
        Ok(())
    } else {
        Err("Caller is not allowed.".to_string())
    }
}


pub fn is_caller_controller() -> Result<(), String> {
    if !is_controller(&caller()) {
        return Err("Unauthorize".into());
    }
    Ok(())
}