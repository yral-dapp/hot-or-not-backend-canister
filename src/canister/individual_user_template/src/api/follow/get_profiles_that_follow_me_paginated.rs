use std::ops::Bound::Included;

use crate::{data_model::CanisterData, CANISTER_DATA};

use candid::Principal;
use shared_utils::canister_specific::individual_user_template::types::{
    error::GetFollowerOrFollowingPageError,
    follow::{FollowEntryDetail, FollowEntryId},
};

pub const MAX_FOLLOW_ENTRIES_PER_PAGE: usize = 10;

#[ic_cdk::query]
#[candid::candid_method(query)]
pub fn get_profiles_that_follow_me_paginated(
    last_index_received: Option<u64>,
) -> Result<Vec<(FollowEntryId, FollowEntryDetail)>, GetFollowerOrFollowingPageError> {
    let current_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();
        get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        )
    })
}

fn get_profiles_that_follow_me_paginated_impl(
    canister_data: &CanisterData,
    last_index_received: Option<u64>,
    current_caller: &Principal,
) -> Result<Vec<(FollowEntryId, FollowEntryDetail)>, GetFollowerOrFollowingPageError> {
    if *current_caller == Principal::anonymous() {
        return Err(GetFollowerOrFollowingPageError::Unauthenticated);
    }

    let my_principal_id = canister_data
        .profile
        .principal_id
        .expect("My principal ID not set");

    if my_principal_id != *current_caller {
        return Err(GetFollowerOrFollowingPageError::Unauthorized);
    }

    let follower = &canister_data.follow_data.follower;
    let last_key: u64 = follower
        .sorted_index
        .last_key_value()
        .map_or(0, |(k, _)| *k);

    Ok(follower
        .sorted_index
        .range((
            Included(0),
            Included(last_index_received.unwrap_or(last_key)),
        ))
        .rev()
        .take(MAX_FOLLOW_ENTRIES_PER_PAGE)
        .map(|(id, entry)| (*id, entry.clone()))
        .collect::<Vec<(u64, FollowEntryDetail)>>())
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_get_profiles_that_follow_me_paginated_impl() {
        let mut canister_data = CanisterData::default();
        let mut current_caller = Principal::anonymous();
        let mut last_index_received: Option<u64> = None;

        let result =
            get_profiles_that_follow_me_paginated_impl(&canister_data, None, &current_caller);

        assert_eq!(
            result,
            Err(GetFollowerOrFollowingPageError::Unauthenticated)
        );

        current_caller = get_mock_user_bob_principal_id();
        canister_data.profile.principal_id = Some(get_mock_user_alice_principal_id());

        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result, Err(GetFollowerOrFollowingPageError::Unauthorized));

        current_caller = get_mock_user_alice_principal_id();

        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result.as_ref().unwrap().len(), 0);

        (0..25).for_each(|id: u64| {
            let follow_entry_detail = FollowEntryDetail {
                principal_id: Principal::self_authenticating(&id.to_ne_bytes()),
                canister_id: Principal::self_authenticating(&id.to_ne_bytes()),
            };
            canister_data.follow_data.follower.add(follow_entry_detail);
        });

        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result.as_ref().unwrap().len(), 10);
        assert_eq!(
            result.unwrap(),
            (15..=24)
                .rev()
                .map(|id: u64| (
                    id,
                    FollowEntryDetail {
                        principal_id: Principal::self_authenticating(&id.to_ne_bytes()),
                        canister_id: Principal::self_authenticating(&id.to_ne_bytes()),
                    }
                ))
                .collect::<Vec<(u64, FollowEntryDetail)>>()
        );

        last_index_received = Some(15);
        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result.as_ref().unwrap().len(), 10);
        assert_eq!(
            result.unwrap(),
            (6..=15)
                .rev()
                .map(|id: u64| (
                    id,
                    FollowEntryDetail {
                        principal_id: Principal::self_authenticating(&id.to_ne_bytes()),
                        canister_id: Principal::self_authenticating(&id.to_ne_bytes()),
                    }
                ))
                .collect::<Vec<(u64, FollowEntryDetail)>>()
        );

        last_index_received = Some(5);

        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result.as_ref().unwrap().len(), 6);
        assert_eq!(
            result.unwrap(),
            (0..=5)
                .rev()
                .map(|id: u64| (
                    id,
                    FollowEntryDetail {
                        principal_id: Principal::self_authenticating(&id.to_ne_bytes()),
                        canister_id: Principal::self_authenticating(&id.to_ne_bytes()),
                    }
                ))
                .collect::<Vec<(u64, FollowEntryDetail)>>()
        );

        last_index_received = Some(0);

        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result.as_ref().unwrap().len(), 1);
        assert_eq!(
            result.unwrap(),
            (0..1)
                .rev()
                .map(|id: u64| (
                    id,
                    FollowEntryDetail {
                        principal_id: Principal::self_authenticating(&id.to_ne_bytes()),
                        canister_id: Principal::self_authenticating(&id.to_ne_bytes()),
                    }
                ))
                .collect::<Vec<(u64, FollowEntryDetail)>>()
        );

        last_index_received = Some(100);

        let result = get_profiles_that_follow_me_paginated_impl(
            &canister_data,
            last_index_received,
            &current_caller,
        );

        assert_eq!(result.as_ref().unwrap().len(), 10);
        assert_eq!(
            result.unwrap(),
            (15..=24)
                .rev()
                .map(|id: u64| (
                    id,
                    FollowEntryDetail {
                        principal_id: Principal::self_authenticating(&id.to_ne_bytes()),
                        canister_id: Principal::self_authenticating(&id.to_ne_bytes()),
                    }
                ))
                .collect::<Vec<(u64, FollowEntryDetail)>>()
        );
    }
}
