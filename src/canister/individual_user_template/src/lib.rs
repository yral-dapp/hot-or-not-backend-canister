use std::cell::RefCell;
use std::{collections::BTreeMap, time::SystemTime};

use api::profile::update_profile_display_details::UpdateProfileDetailsError;
use candid::{Nat, Principal};
use data_model::pump_n_dump::TokenBetGame;
use data_model::CanisterData;
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::export_candid;
use ic_nns_governance::pb::v1::{
    SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse,
};
use ic_sns_init::pb::v1::SnsInitPayload;
use icrc_ledger_types::icrc1::transfer::Memo;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::{BetMakerArg, FolloweeArg, IndividualUserTemplateInitArgs, PlaceBetArg},
        cdao::DeployedCdaoCanisters,
        device_id::DeviceIdentity,
        error::{
            AirdropError, BetOnCurrentlyViewingPostError, CdaoDeployError, CdaoTokenError,
            FollowAnotherUserProfileError, GetPostsOfUserProfileError,
        },
        follow::{FollowEntryDetail, FollowEntryId},
        hot_or_not::{BetDetails, BetOutcomeForBetMaker, BettingStatus, PlacedBetDetail},
        kv_storage::{NamespaceErrors, NamespaceForFrontend},
        migration::MigrationErrors,
        ml_data::{MLFeedCacheItem, SuccessHistoryItemV1, WatchHistoryItem},
        post::{
            Post, PostDetailsForFrontend, PostDetailsFromFrontend, PostViewDetailsFromFrontend,
        },
        profile::{
            UserCanisterDetails, UserProfileDetailsForFrontend, UserProfileDetailsForFrontendV2,
            UserProfileUpdateDetailsFromFrontend,
        },
        pump_n_dump::{BalanceInfo, ParticipatedGameInfo, PumpNDumpStateDiff, PumpsAndDumps},
        session::SessionType,
    },
    common::types::{
        app_primitive_type::PostId,
        http::{HttpRequest, HttpResponse},
        known_principal::KnownPrincipalType,
        top_posts::post_score_index_item::PostStatus,
        utility_token::token_event::TokenEvent,
    },
    pagination::PaginationError,
    types::canister_specific::individual_user_template::error_types::{
        GetUserUtilityTokenTransactionHistoryError, UpdateProfileSetUniqueUsernameError,
    },
    types::creator_dao_stats::IndividualUserCreatorDaoEntry,
};

mod api;
pub mod data_model;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
    static SNAPSHOT_DATA: RefCell<Vec<u8>> = RefCell::default();
    static PUMP_N_DUMP: RefCell<TokenBetGame> = RefCell::default();
}

export_candid!();
