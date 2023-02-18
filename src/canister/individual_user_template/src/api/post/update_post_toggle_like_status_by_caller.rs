use crate::CANISTER_DATA;
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::date_time::system_time;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_post_toggle_like_status_by_caller(id: u64) -> bool {
    let caller_id = SPrincipal(ic_cdk::caller());

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        let updated_like_status = post_to_update
            .toggle_like_status(&caller_id, &system_time::get_current_system_time_from_ic);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);

        updated_like_status
    })
}
