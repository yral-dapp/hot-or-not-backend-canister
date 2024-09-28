// For feed_filter_upgrade_test
// #[ic_cdk::update]
// #[candid::candid_method(update)]
// fn delete_post_temp(id: u64) {
//     CANISTER_DATA.with(|canister_data_ref_cell| {
//         let api_caller = ic_cdk::caller();

//         let global_super_admin_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
//             canister_data_ref_cell
//                 .borrow()
//                 .known_principal_ids
//                 .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
//                 .cloned()
//                 .unwrap()
//         });

//         if api_caller != global_super_admin_principal_id {
//             return;
//         }

//         canister_data_ref_cell
//             .borrow_mut()
//             .all_created_posts
//             .remove(&id);
//     });
// }
