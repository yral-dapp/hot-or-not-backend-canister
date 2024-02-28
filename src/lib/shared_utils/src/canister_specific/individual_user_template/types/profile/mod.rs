use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(Default, Clone, CandidType, Deserialize, Debug, Serialize)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub unique_user_name: Option<String>,
    pub principal_id: Option<Principal>,
    pub profile_picture_url: Option<String>,
    pub profile_stats: UserProfileGlobalStats,
}

#[derive(CandidType, Deserialize, Debug, PartialEq, Eq)]
pub struct UserProfileDetailsForFrontend {
    pub display_name: Option<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub principal_id: Principal,
    pub profile_picture_url: Option<String>,
    pub profile_stats: UserProfileGlobalStats,
    pub lifetime_earnings: u64,
    pub unique_user_name: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct UserProfileGlobalStats {
    pub hot_bets_received: u64,
    pub not_bets_received: u64,
}

#[derive(Deserialize, CandidType)]
pub struct UserProfileUpdateDetailsFromFrontend {
    pub display_name: Option<String>,
    pub profile_picture_url: Option<String>,
}
