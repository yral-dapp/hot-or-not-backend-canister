// use candid::Principal;
// use shared_utils::{
//     canister_specific::individual_user_template::types::hot_or_not::BettingStatus,
//     common::utils::system_time::{self, SystemTimeProvider},
// };

// use crate::{data_model::CanisterData, CANISTER_DATA};

// #[ic_cdk::query]
// #[candid::candid_method(query)]
// fn get_my_bet_details_for_a_hot_or_not_post(canister_id: Principal, post_id: u64) -> BettingStatus {
//     CANISTER_DATA.with(|canister_data_ref_cell| {
//         get_my_bet_details_for_a_hot_or_not_post_impl(
//             &canister_data_ref_cell.borrow(),
//             &system_time::get_current_system_time_from_ic,
//             canister_id,
//             post_id,
//         )
//     })
// }

// fn get_my_bet_details_for_a_hot_or_not_post_impl(
//     canister_data: &CanisterData,
//     time_provider: &SystemTimeProvider,
//     canister_id: Principal,
//     post_id: u64,
// ) -> BettingStatus {
//     canister_data
//         .all_created_posts
//         .get(&post_id)
//         .unwrap()
//         .get_hot_or_not_betting_status_for_this_post(time_provider)
// }

// #[cfg(test)]
// mod test {
//     use std::{
//         collections::HashSet,
//         time::{Duration, SystemTime},
//     };

//     use shared_utils::{
//         canister_specific::individual_user_template::types::post::{
//             HotOrNotDetails, Post, PostViewStatistics,
//         },
//         types::canister_specific::individual_user_template::post::PostStatus,
//     };

//     use super::*;

//     #[test]
//     fn test_get_my_bet_details_for_a_hot_or_not_post_impl() {}
// }
