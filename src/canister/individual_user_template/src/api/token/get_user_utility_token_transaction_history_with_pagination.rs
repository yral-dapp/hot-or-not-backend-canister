use crate::CANISTER_DATA;
use shared_utils::{
    common::types::utility_token::token_event::TokenEvent,
    pagination::{self, PaginationError},
    types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
};

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_user_utility_token_transaction_history_with_pagination(
    from_inclusive_id: u64,
    to_exclusive_id: u64,
) -> Result<Vec<(u64, TokenEvent)>, GetUserUtilityTokenTransactionHistoryError> {
    let (from_inclusive_id, to_exclusive_id) = pagination::get_pagination_bounds(
        from_inclusive_id,
        to_exclusive_id,
        CANISTER_DATA.with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow()
                .my_token_balance
                .utility_token_transaction_history
                .len()
        }) as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => {
            GetUserUtilityTokenTransactionHistoryError::InvalidBoundsPassed
        }
        PaginationError::ReachedEndOfItemsList => {
            GetUserUtilityTokenTransactionHistoryError::ReachedEndOfItemsList
        }
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            GetUserUtilityTokenTransactionHistoryError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    Ok(CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .my_token_balance
            .utility_token_transaction_history
            .iter()
            .rev()
            .skip(from_inclusive_id as usize)
            .take((to_exclusive_id - from_inclusive_id) as usize)
            .map(|(time, token_event)| (*time, token_event.clone()))
            .collect()
    }))
}
