use std::cell::RefCell;
use std::time::SystemTime;

use api::{
    follow::update_profiles_that_follow_me_toggle_list_with_specified_profile::FollowerArg,
    profile::update_profile_display_details::UpdateProfileDetailsError,
};
use candid::Principal;
use data_model::CanisterData;
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::export_candid;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        session::SessionType,
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
pub mod data_model;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
    static SNAPSHOT_DATA: RefCell<Vec<u8>> = RefCell::default();
}

export_candid!();
