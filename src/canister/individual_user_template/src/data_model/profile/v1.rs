use candid::{CandidType, Deserialize, Principal};
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileGlobalStats;

use super::v0;

#[derive(Default, Clone, CandidType, Deserialize)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub unique_user_name: Option<String>,
    pub principal_id: Option<Principal>,
    pub profile_picture_url: Option<String>,
    pub profile_stats: UserProfileGlobalStats,
}

impl From<v0::UserProfile> for UserProfile {
    fn from(value: v0::UserProfile) -> Self {
        Self {
            display_name: value.display_name,
            unique_user_name: value.unique_user_name,
            principal_id: Some(value.principal_id.0),
            profile_picture_url: value.profile_picture_url,
            profile_stats: value.profile_stats,
        }
    }
}
