use std::str::FromStr;

use candid::Principal;
use ic_cdk::storage;
use shared_utils::{common::types::known_principal::KnownPrincipalType, constant::GOVERNANCE_CANISTER_ID};

use crate::CANISTER_DATA;

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok((canister_data,)) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
                canister_data_ref_cell.borrow_mut().known_principal_ids.insert(KnownPrincipalType::CanisterIdSnsGovernance, Principal::from_str(GOVERNANCE_CANISTER_ID).unwrap());
            });
        }
        Err(_) => {
            panic!("Failed to restore canister data from stable memory");
        }
    }
}
