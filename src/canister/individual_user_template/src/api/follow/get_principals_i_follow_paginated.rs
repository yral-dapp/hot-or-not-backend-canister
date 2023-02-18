use candid::{CandidType, Principal};
use shared_utils::pagination::{self, PaginationError};

use crate::CANISTER_DATA;

#[derive(CandidType)]
pub enum GetFollowerOrFollowingError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}

#[ic_cdk::query]
#[candid::candid_method(query)]
pub fn get_principals_i_follow_paginated(
    from_inclusive_index: u64,
    to_exclusive_index: u64,
) -> Result<Vec<Principal>, GetFollowerOrFollowingError> {
    let (from_inclusive_index, to_exclusive_index) = pagination::get_pagination_bounds(
        from_inclusive_index,
        to_exclusive_index,
        CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell.borrow().principals_i_follow.len()
        }) as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => GetFollowerOrFollowingError::InvalidBoundsPassed,
        PaginationError::ReachedEndOfItemsList => {
            GetFollowerOrFollowingError::ReachedEndOfItemsList
        }
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            GetFollowerOrFollowingError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    Ok(CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .principals_i_follow
            .iter()
            .skip(from_inclusive_index as usize)
            .take(to_exclusive_index as usize)
            .cloned()
            .collect()
    }))
}
