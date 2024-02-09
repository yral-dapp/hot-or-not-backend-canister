use std::borrow::Borrow;

use candid::candid_method;

use crate::CANISTER_DATA;


#[ic_cdk::query]
#[candid_method(query)]
pub fn get_subnet_available_capacity() -> u64 {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.borrow().available_canisters.len() as u64
    })
}