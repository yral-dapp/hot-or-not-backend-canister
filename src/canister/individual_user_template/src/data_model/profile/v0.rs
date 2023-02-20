use candid::CandidType;
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileGlobalStats;

#[derive(CandidType)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub unique_user_name: Option<String>,
    pub principal_id: SPrincipal,
    pub profile_picture_url: Option<String>,
    pub profile_stats: UserProfileGlobalStats,
}
