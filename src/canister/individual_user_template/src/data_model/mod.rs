use std::{
    collections::{BTreeMap, BTreeSet},
    time::SystemTime,time::UNIX_EPOCH
};

use candid::{Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_stable_structures::Storable;

use std::borrow::Cow;
use ic_stable_structures::storable::Bound;

// use ic_cdk_timers::TimerId;
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        configuration::IndividualUserConfiguration,
        follow::FollowData,
        hot_or_not::{
            BetDetails, GlobalBetId, GlobalRoomId, PlacedBetDetail, RoomDetailsV1, RoomId,
            SlotDetailsV1,  StablePrincipal,
        },
        migration::MigrationInfo,
        post::{FeedScore, Post, PostViewStatistics},
        profile::UserProfile,
        session::SessionType,
        token::TokenBalance,
    },
    common::types::{
        app_primitive_type::PostId, known_principal::KnownPrincipalMap, top_posts::{post_score_index::PostScoreIndex, post_score_index_item::PostStatus}, utility_token::token_event::{NewSlotType, SystemTimeInMs}, version_details::VersionDetails
    },
};

use self::memory::{
    get_bet_details_memory,
    // get_global_bet_timer_memory,
    get_bet_timer_first_bet_at_memory,
    get_bet_timer_memory,
    get_post_principal_memory,
    get_room_details_memory,
    get_slot_details_memory,
    Memory,
};

use kv_storage::AppStorage;

pub mod kv_storage;
pub mod memory;

#[derive(Deserialize, Serialize)]
pub struct CanisterData {
    // Key is Post ID
    pub all_created_posts: BTreeMap<u64, Post>,
    #[serde(skip, default = "_default_room_details")]
    pub room_details_map:
        ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory>,
    #[serde(skip, default = "_default_bet_details")]
    pub bet_details_map: ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory>,
    #[serde(skip, default = "_default_post_principal_map")]
    pub post_principal_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory>,
    #[serde(skip, default = "_default_slot_details_map")]
    pub slot_details_map:
        ic_stable_structures::btreemap::BTreeMap<(PostId, NewSlotType), SlotDetailsV1, Memory>,
    pub all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail>,
    pub configuration: IndividualUserConfiguration,
    pub follow_data: FollowData,
    pub known_principal_ids: KnownPrincipalMap,
    pub my_token_balance: TokenBalance,
    pub posts_index_sorted_by_home_feed_score: PostScoreIndex,
    pub posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex,
    pub principals_i_follow: BTreeSet<Principal>,
    pub principals_that_follow_me: BTreeSet<Principal>,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    #[serde(default)]
    pub session_type: Option<SessionType>,
    #[serde(default)]
    pub last_access_time: Option<SystemTime>,
    #[serde(default)]
    pub last_canister_functionality_access_time: Option<SystemTime>,
    #[serde(default)]
    pub migration_info: MigrationInfo,
    #[serde(default)]
    pub app_storage: AppStorage,
    // u64 is post_id, SystemTime refers to time when first_bet is placed
    #[serde(skip, default = "_default_bet_timer_vec")]
    pub bet_timer_posts: ic_stable_structures::vec::Vec<PostId, Memory>,
    // this keeps track of when the first bet was placed.
    #[serde(skip, default = "_default_bet_timer_first_bet_placed_at_map")]
    // pub first_bet_placed_at_hashmap: BTreeMap<PostId, SystemTime>,
    pub first_bet_placed_at_hashmap:
        ic_stable_structures::btreemap::BTreeMap<PostId, SystemTimeInMs, Memory>,
    // #{serde(skip, default = "_default_global_bet_timer")}
    // there is one global timer for processing bets
    // pub global_bet_timer: Option<TimerId>,
}

pub fn _default_room_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalRoomId, RoomDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_room_details_memory())
}

pub fn _default_bet_details(
) -> ic_stable_structures::btreemap::BTreeMap<GlobalBetId, BetDetails, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_details_memory())
}

pub fn _default_post_principal_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, StablePrincipal), (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_post_principal_memory())
}

pub fn _default_slot_details_map(
) -> ic_stable_structures::btreemap::BTreeMap<(PostId, NewSlotType), SlotDetailsV1, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_slot_details_memory())
}

pub fn _default_bet_timer_first_bet_placed_at_map(
) -> ic_stable_structures::btreemap::BTreeMap<PostId, SystemTimeInMs, Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_bet_timer_first_bet_at_memory())
}

pub fn _default_bet_timer_vec() -> ic_stable_structures::vec::Vec<PostId, Memory> {
    match ic_stable_structures::vec::Vec::init(get_bet_timer_memory()) {
        Ok(vec) => vec,
        Err(err) => panic!("Failed to initialize bet timer vec: {}", err),
    }
}

// pub fn _default_global_bet_timer() -> ic_stable_structures::vec::Vec<PostId, Memory> {
//     match ic_stable_structures::vec::Vec::init(get_global_bet_timer_memory()) {
//         Ok(vec) => vec,
//         Err(err) => panic!("Failed to initialize bet timer vec: {}", err),
//     }
// }

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            all_created_posts: BTreeMap::new(),
            room_details_map: _default_room_details(),
            bet_details_map: _default_bet_details(),
            post_principal_map: _default_post_principal_map(),
            slot_details_map: _default_slot_details_map(),
            all_hot_or_not_bets_placed: BTreeMap::new(),
            configuration: IndividualUserConfiguration::default(),
            follow_data: FollowData::default(),
            known_principal_ids: KnownPrincipalMap::default(),
            my_token_balance: TokenBalance::default(),
            posts_index_sorted_by_home_feed_score: PostScoreIndex::default(),
            posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndex::default(),
            principals_i_follow: BTreeSet::new(),
            principals_that_follow_me: BTreeSet::new(),
            profile: UserProfile::default(),
            version_details: VersionDetails::default(),
            session_type: None,
            last_access_time: None,
            last_canister_functionality_access_time: None,
            migration_info: MigrationInfo::NotMigrated,
            app_storage: AppStorage::default(),
            // these two fields together help with infinite slots in yral game
            first_bet_placed_at_hashmap: _default_bet_timer_first_bet_placed_at_map(),
            bet_timer_posts: _default_bet_timer_vec(),
        }
    }
}


// impl Storable for SystemTime {
//     fn to_bytes(&self) -> Cow<[u8]> {
//         let duration_since_epoch = self.duration_since(UNIX_EPOCH).expect("Time went backwards");
//         let secs = duration_since_epoch.as_secs();
//         let nanos = duration_since_epoch.subsec_nanos();
//         let mut bytes = Vec::with_capacity(12);
//         bytes.extend(&secs.to_le_bytes());
//         bytes.extend(&nanos.to_le_bytes());
//         Cow::Owned(bytes)
//     }

//     fn from_bytes(bytes: Cow<[u8]>) -> Self {
//         let secs = u64::from_le_bytes(bytes[0..8].try_into().expect("slice with incorrect length"));
//         let nanos = u32::from_le_bytes(bytes[8..12].try_into().expect("slice with incorrect length"));
//         UNIX_EPOCH + std::time::Duration::new(secs, nanos)
//     }

//     const BOUND: Bound = Bound::Bounded {
//         max_size: 12,
//         is_fixed_size: true,
//     };
// }
