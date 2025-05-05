use ic_cdk_macros::query;
use shared_utils::types::creator_dao_stats::CreatorDaoTokenStats;

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[query(guard = "is_caller_platform_global_admin_or_controller")]
pub fn get_creator_dao_stats() -> CreatorDaoTokenStats {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.creator_dao_stats.clone())
}
