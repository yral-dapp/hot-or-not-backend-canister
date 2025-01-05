use ic_cdk::{api::is_controller, caller};
use candid::Principal;
use crate::constant::{GLOBAL_SUPER_ADMIN_USER_ID, GLOBAL_SUPER_ADMIN_USER_ID_V1, GOVERNANCE_CANISTER_ID, RECLAIM_CANISTER_PRINCIPAL_ID};

pub fn is_reclaim_canister_id() -> Result<(), String> {
    let caller = ic_cdk::caller();

    let valid_principals = vec![RECLAIM_CANISTER_PRINCIPAL_ID, GLOBAL_SUPER_ADMIN_USER_ID_V1];


    if valid_principals.contains(&caller.to_string().as_ref()) {
        Ok(())
    } else {
        Err("Caller is not allowed.".to_string())
    }
}


pub fn is_caller_global_admin() -> Result<(), String> {

    let valid_canisters = vec![GLOBAL_SUPER_ADMIN_USER_ID_V1, GLOBAL_SUPER_ADMIN_USER_ID];


    if !valid_canisters.contains(&caller().to_string().as_str()){
        return Err("Unauthorize".into())
    }
    Ok(())
}


pub fn is_caller_controller() -> Result<(), String> {
    if !is_controller(&caller()) {
        return Err("Unauthorize".into());
    }
    Ok(())
}


pub fn is_caller_controller_or_global_admin() -> Result<(), String> {
    if !is_controller(&caller()) && !caller().to_string().eq(GLOBAL_SUPER_ADMIN_USER_ID) {
        return Err("Unauthorize".into())
    }

    Ok(())
}


pub fn is_caller_governance_canister() -> Result<(), String> {
    if !caller().to_string().eq(GOVERNANCE_CANISTER_ID) {
        return Err("Unauthorized".into());
    }

    Ok(())
}