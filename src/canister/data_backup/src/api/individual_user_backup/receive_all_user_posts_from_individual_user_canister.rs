use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::post::Post,
    common::types::storable_principal::StorablePrincipal,
};

use crate::{data::memory_layout::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn receive_all_user_posts_from_individual_user_canister(
    all_user_posts_from_individual_user_canister_vec: Vec<Post>,
    canister_owner_principal_id: Principal,
) {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        receive_all_user_posts_from_individual_user_canister_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            all_user_posts_from_individual_user_canister_vec,
            &caller_principal_id,
            &canister_owner_principal_id,
        );
    });
}

fn receive_all_user_posts_from_individual_user_canister_impl(
    canister_data: &mut CanisterData,
    all_user_posts_from_individual_user_canister: Vec<Post>,
    caller_principal_id: &Principal,
    canister_owner_principal_id: &Principal,
) {
    let does_the_current_call_makers_record_exist = canister_data
        .user_principal_id_to_all_user_data_map
        .contains_key(&StorablePrincipal(*canister_owner_principal_id));

    if !does_the_current_call_makers_record_exist {
        return;
    }

    let mut existing_entry = canister_data
        .user_principal_id_to_all_user_data_map
        .get(&StorablePrincipal(*canister_owner_principal_id))
        .unwrap();

    if existing_entry.user_canister_id != *caller_principal_id {
        return;
    }

    all_user_posts_from_individual_user_canister
        .iter()
        .for_each(|post| {
            // upsert the post details in the user's record.
            existing_entry
                .canister_data
                .all_created_posts
                .insert(post.id, post.clone());
        });

    canister_data.user_principal_id_to_all_user_data_map.insert(
        StorablePrincipal(*canister_owner_principal_id),
        existing_entry,
    );
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, time::SystemTime};

    use shared_utils::canister_specific::{
        data_backup::types::all_user_data::{AllUserData, UserOwnedCanisterData},
        individual_user_template::types::{
            hot_or_not::HotOrNotDetails,
            post::{FeedScore, PostStatus, PostViewStatistics},
        },
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_canister_id,
    };

    use super::*;

    #[test]
    fn test_receive_all_user_posts_from_individual_user_canister_impl() {
        let mut canister_data = CanisterData::default();

        let all_user_posts_from_individual_user_canister = vec![
            Post {
                id: 0,
                description: "alice post 0 - description".to_string(),
                hashtags: ["alice-tag-0".to_string(), "alice-tag-1".to_string()].to_vec(),
                video_uid: "alice-video-0".to_string(),
                status: PostStatus::Uploaded,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics {
                    total_view_count: 1,
                    threshold_view_count: 0,
                    average_watch_percentage: 0,
                },
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: Some(HotOrNotDetails::default()),
            },
            Post {
                id: 1,
                description: "alice post 1 - description".to_string(),
                hashtags: ["alice-tag-2".to_string(), "alice-tag-3".to_string()].to_vec(),
                video_uid: "alice-video-1".to_string(),
                status: PostStatus::Uploaded,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics {
                    total_view_count: 1,
                    threshold_view_count: 0,
                    average_watch_percentage: 0,
                },
                home_feed_score: FeedScore::default(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: Some(HotOrNotDetails::default()),
            },
        ];

        receive_all_user_posts_from_individual_user_canister_impl(
            &mut canister_data,
            all_user_posts_from_individual_user_canister.clone(),
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_none());

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_bob_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_all_user_posts_from_individual_user_canister_impl(
            &mut canister_data,
            all_user_posts_from_individual_user_canister.clone(),
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_some());
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .all_created_posts
                .len(),
            0
        );

        canister_data.user_principal_id_to_all_user_data_map.insert(
            StorablePrincipal(get_mock_user_alice_principal_id()),
            AllUserData {
                user_principal_id: get_mock_user_alice_principal_id(),
                user_canister_id: get_mock_user_alice_canister_id(),
                canister_data: UserOwnedCanisterData::default(),
            },
        );

        receive_all_user_posts_from_individual_user_canister_impl(
            &mut canister_data,
            all_user_posts_from_individual_user_canister,
            &get_mock_user_alice_canister_id(),
            &get_mock_user_alice_principal_id(),
        );

        assert!(canister_data
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
            .is_some());
        assert_eq!(
            canister_data
                .user_principal_id_to_all_user_data_map
                .get(&StorablePrincipal(get_mock_user_alice_principal_id()))
                .unwrap()
                .canister_data
                .all_created_posts
                .len(),
            2
        );
    }
}
