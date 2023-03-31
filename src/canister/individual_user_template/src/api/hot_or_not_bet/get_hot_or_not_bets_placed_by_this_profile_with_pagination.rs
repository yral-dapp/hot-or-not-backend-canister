use shared_utils::canister_specific::individual_user_template::types::hot_or_not::PlacedBetDetail;

use crate::CANISTER_DATA;

const PAGINATION_PAGE_SIZE: usize = 10;

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_hot_or_not_bets_placed_by_this_profile_with_pagination(
    last_index_sent: usize,
) -> Vec<PlacedBetDetail> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .iter()
            .skip(last_index_sent)
            .take(PAGINATION_PAGE_SIZE)
            .map(|(_, placed_bet_detail)| placed_bet_detail.clone())
            .collect()
    })
}
