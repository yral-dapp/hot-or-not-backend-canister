use ic_cdk_macros::query;

use crate::CANISTER_DATA;

mod get_all_available_subnet_orchestrators;
mod get_all_subnet_orchestrators;
mod get_last_subnet_upgrade_status;
mod global_admin;
mod known_principal;
mod populate_known_principal_for_all_subnet;
pub mod provision_subnet_orchestrator;
mod recharge_subnet_orchestrator;
mod reinstall_yral_post_cache_canister;
pub mod remove_subnet_orchestrator_from_available_list;
mod stop_upgrades_for_individual_user_canisters;
mod subnet_orchestrator_maxed_out;
mod update_canisters_last_access_time;
mod update_profile_owner_for_individual_users;
pub mod upgrade_canisters_in_network;
mod upgrade_specific_individual_canister;
pub mod upload_wasms;
pub mod update_timers_for_hon_game;

#[query]
pub fn get_version() -> String {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.version_detail.version.clone())
}
