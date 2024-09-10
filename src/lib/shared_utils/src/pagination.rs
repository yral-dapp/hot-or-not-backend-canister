use candid::CandidType;

use crate::constant::MAX_POSTS_IN_ONE_REQUEST;

#[derive(PartialEq, Debug, CandidType)]
pub enum PaginationError {
    InvalidBoundsPassed,
    ReachedEndOfItemsList,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}

pub fn get_pagination_bounds(
    from_inclusive_id: u64,
    to_exclusive_id: u64,
    total_items: u64,
) -> Result<(u64, u64), PaginationError> {
    let mut upper_bound_exclusive = to_exclusive_id;

    if to_exclusive_id > total_items {
        upper_bound_exclusive = total_items;
    }

    if from_inclusive_id >= to_exclusive_id {
        return Err(PaginationError::InvalidBoundsPassed);
    }

    if from_inclusive_id >= total_items {
        return Err(PaginationError::ReachedEndOfItemsList);
    }

    if (to_exclusive_id - from_inclusive_id) > MAX_POSTS_IN_ONE_REQUEST {
        return Err(PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest);
    }

    Ok((from_inclusive_id, upper_bound_exclusive))
}

pub fn get_pagination_bounds_cursor(
    from_inclusive_id: u64,
    limit: u64,
    total_items: u64,
) -> Result<(u64, u64), PaginationError> {
    if from_inclusive_id >= total_items {
        return Err(PaginationError::ReachedEndOfItemsList);
    }

    let mut adjusted_limit = limit;
    if adjusted_limit > total_items - from_inclusive_id {
        adjusted_limit = total_items - from_inclusive_id;
    }

    if adjusted_limit > MAX_POSTS_IN_ONE_REQUEST {
        return Err(PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest);
    }

    Ok((from_inclusive_id, adjusted_limit))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_pagination_bounds() {
        // exact number of items as upper bound
        assert_eq!(get_pagination_bounds(0, 10, 10), Ok((0, 10)));

        // upper bound exceeds total items
        assert_eq!(get_pagination_bounds(0, 10, 5), Ok((0, 5)));

        // total items exceeds upper bound
        assert_eq!(get_pagination_bounds(0, 10, 15), Ok((0, 10)));

        // lower bound exceeds upper bound
        assert_eq!(
            get_pagination_bounds(10, 0, 15),
            Err(PaginationError::InvalidBoundsPassed)
        );

        // number of items is zero
        assert_eq!(
            get_pagination_bounds(0, 10, 0),
            Err(PaginationError::ReachedEndOfItemsList)
        );

        // lower bound exceeds total items
        assert_eq!(
            get_pagination_bounds(10, 20, 5),
            Err(PaginationError::ReachedEndOfItemsList)
        );

        // number of items fetched exceeds max allowed
        assert_eq!(
            get_pagination_bounds(0, 110, 250),
            Err(PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest)
        );
    }

    #[test]
    fn test_get_pagination_bounds_cursor() {
        // exact number of items as limit
        assert_eq!(get_pagination_bounds_cursor(0, 10, 10), Ok((0, 10)));

        // limit exceeds total items
        assert_eq!(get_pagination_bounds_cursor(0, 10, 5), Ok((0, 5)));

        // total items exceeds limit
        assert_eq!(get_pagination_bounds_cursor(0, 10, 15), Ok((0, 10)));

        // number of items is zero
        assert_eq!(
            get_pagination_bounds_cursor(0, 10, 0),
            Err(PaginationError::ReachedEndOfItemsList)
        );

        // lower bound exceeds total items
        assert_eq!(
            get_pagination_bounds_cursor(10, 10, 5),
            Err(PaginationError::ReachedEndOfItemsList)
        );

        // number of items fetched exceeds max allowed
        assert_eq!(
            get_pagination_bounds_cursor(0, 110, 250),
            Err(PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest)
        );
    }
}
