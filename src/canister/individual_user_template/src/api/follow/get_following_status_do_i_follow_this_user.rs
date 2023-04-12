use candid::Principal;

use crate::CANISTER_DATA;

#[deprecated]
#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_following_status_do_i_follow_this_user(user_principal_to_check: Principal) -> bool {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .principals_i_follow
            .contains(&user_principal_to_check)
    })
}
