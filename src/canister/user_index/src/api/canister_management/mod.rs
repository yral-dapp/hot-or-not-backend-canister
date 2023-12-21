use ic_cdk::api::{management_canister::{main::{canister_status, CanisterStatusResponse}, provisional::CanisterIdRecord}, call::CallResult};
use candid::Principal;



#[candid::candid_method(update)]
#[ic_cdk::update]
pub async fn get_user_canister_status(canister_id: Principal) -> CallResult<(CanisterStatusResponse,)>{
    canister_status(CanisterIdRecord {canister_id}).await
}