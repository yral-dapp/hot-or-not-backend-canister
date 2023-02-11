use candid::CandidType;
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileGlobalStats;
use speedy::{Readable, Writable};

#[derive(Readable, Writable, CandidType)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub unique_user_name: Option<String>,
    pub principal_id: SPrincipal,
    pub profile_picture_url: Option<String>,
    pub profile_stats: UserProfileGlobalStats,
}

impl UserProfile {
    // pub fn new(principal_id: Principal) -> Self {
    //     // let generated_name = generate_random_names(principal_id).await;

    //     Self {
    //         display_name: None,
    //         unique_user_name: None,
    //         principal_id: SPrincipal(principal_id),
    //         profile_picture_url: None,
    //         profile_stats: UserProfileGlobalStats {
    //             lifetime_earnings: 0,
    //             hots_earned_count: 0,
    //             nots_earned_count: 0,
    //         },
    //     }
    // }

    // pub fn update_profile_details(
    //     &mut self,
    //     user_profile_details: UserProfileUpdateDetailsFromFrontend,
    // ) {
    //     self.display_name = user_profile_details.display_name;
    //     self.profile_picture_url = user_profile_details.profile_picture_url;
    // }

    // pub fn set_unique_user_name(&mut self, unique_user_name: String) {
    //     self.unique_user_name = Some(unique_user_name);
    // }
}
