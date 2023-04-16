use std::cell::RefCell;

use api::{
    follow::update_profiles_that_follow_me_toggle_list_with_specified_profile::FollowerArg,
    profile::update_profile_display_details::UpdateProfileDetailsError,
};
use candid::{export_service, Principal};
use data_model::CanisterData;
use ic_cdk::api::management_canister::provisional::CanisterId;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::{FolloweeArg, IndividualUserTemplateInitArgs, PlaceBetArg},
        error::{
            BetOnCurrentlyViewingPostError, FollowAnotherUserProfileError,
            GetPostsOfUserProfileError,
        },
        follow::{FollowEntryDetail, FollowEntryId},
        hot_or_not::{BetOutcomeForBetMaker, BettingStatus, PlacedBetDetail},
        post::{
            Post, PostDetailsForFrontend, PostDetailsFromFrontend, PostViewDetailsFromFrontend,
        },
        profile::{
            UserProfile, UserProfileDetailsForFrontend, UserProfileUpdateDetailsFromFrontend,
        },
    },
    common::types::{
        app_primitive_type::PostId, known_principal::KnownPrincipalType,
        utility_token::token_event::TokenEvent,
    },
    types::canister_specific::individual_user_template::error_types::{
        GetUserUtilityTokenTransactionHistoryError, UpdateProfileSetUniqueUsernameError,
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
