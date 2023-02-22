use std::cell::RefCell;

use api::{
    follow::{
        update_principals_i_follow_toggle_list_with_principal_specified::FollowAnotherUserProfileError,
        update_principals_that_follow_me_toggle_list_with_specified_principal::AnotherUserFollowedMeError,
    },
    profile::update_profile_display_details::UpdateProfileDetailsError,
};
use candid::{export_service, Principal};
use data_model::CanisterData;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        args::IndividualUserTemplateInitArgs,
        error::{GetFollowerOrFollowingError, GetPostsOfUserProfileError},
        post::{Post, PostViewDetailsFromFrontend},
        profile::{
            UserProfile, UserProfileDetailsForFrontend, UserProfileUpdateDetailsFromFrontend,
        },
    },
    common::types::known_principal::KnownPrincipalType,
    types::{
        canister_specific::individual_user_template::{
            error_types::{
                GetUserUtilityTokenTransactionHistoryError, UpdateProfileSetUniqueUsernameError,
            },
            post::PostDetailsForFrontend,
        },
        post::PostDetailsFromFrontend,
        utility_token::token_event::TokenEvent,
    },
};

mod api;
mod data_model;
#[cfg(test)]
mod test;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
