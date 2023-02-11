use crate::CANISTER_DATA;

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_utility_token_balance() -> u64 {
    // let my_token_balance = s!(MyTokenBalance);

    // my_token_balance.get_utility_token_balance()
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .my_token_balance
            .utility_token_balance
    })
}
