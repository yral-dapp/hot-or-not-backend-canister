use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Debug)]
pub struct UserProfileDetailsForFrontend {
    pub principal_id: Principal,
    pub display_name: Option<String>,
    pub unique_user_name: Option<String>,
    pub profile_picture_url: Option<String>,
    pub profile_stats: UserProfileGlobalStats,
    pub followers_count: u64,
    pub following_count: u64,
}

#[derive(CandidType, Deserialize, Clone, Copy, Debug, Default)]
pub struct UserProfileGlobalStats {
    pub lifetime_earnings: u64,
    pub hots_earned_count: u64,
    pub nots_earned_count: u64,
}

#[derive(Deserialize, CandidType)]
pub struct UserProfileUpdateDetailsFromFrontend {
    pub display_name: Option<String>,
    pub profile_picture_url: Option<String>,
}
