import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type AnotherUserFollowedMeError = {
    'UserIndexCrossCanisterCallFailed' : null
  } |
  { 'FollowersListFull' : null } |
  { 'NotAuthorized' : null } |
  { 'UserTryingToFollowMeDoesNotExist' : null };
export type FollowAnotherUserProfileError = {
    'UserToFollowDoesNotExist' : null
  } |
  { 'UserIndexCrossCanisterCallFailed' : null } |
  { 'UserITriedToFollowCrossCanisterCallFailed' : null } |
  { 'UsersICanFollowListIsFull' : null } |
  {
    'MyCanisterIDDoesNotMatchMyPrincipalCanisterIDMappingSeenByUserITriedToFollow' : null
  } |
  { 'UserITriedToFollowDidNotFindMe' : null } |
  { 'NotAuthorized' : null } |
  { 'UserITriedToFollowHasTheirFollowersListFull' : null };
export type GetFollowerOrFollowingError = { 'ReachedEndOfItemsList' : null } |
  { 'InvalidBoundsPassed' : null } |
  { 'ExceededMaxNumberOfItemsAllowedInOneRequest' : null };
export type GetPostsOfUserProfileError = { 'ReachedEndOfItemsList' : null } |
  { 'InvalidBoundsPassed' : null } |
  { 'ExceededMaxNumberOfItemsAllowedInOneRequest' : null };
export interface IndividualUserTemplateInitArgs {
  'known_principal_ids' : [] | [Array<[KnownPrincipalType, Principal]>],
  'profile_owner' : [] | [Principal],
}
export type KnownPrincipalType = { 'CanisterIdUserIndex' : null } |
  { 'CanisterIdConfiguration' : null } |
  { 'CanisterIdProjectMemberIndex' : null } |
  { 'CanisterIdTopicCacheIndex' : null } |
  { 'CanisterIdRootCanister' : null } |
  { 'CanisterIdDataBackup' : null } |
  { 'CanisterIdPostCache' : null } |
  { 'CanisterIdSNSController' : null } |
  { 'UserIdGlobalSuperAdmin' : null };
export type MintEvent = {
    'NewUserSignup' : { 'new_user_principal_id' : Principal }
  } |
  {
    'Referral' : {
      'referrer_user_principal_id' : Principal,
      'referee_user_principal_id' : Principal,
    }
  };
export interface PostDetailsForFrontend {
  'id' : bigint,
  'status' : PostStatus,
  'home_feed_ranking_score' : bigint,
  'hashtags' : Array<string>,
  'like_count' : bigint,
  'description' : string,
  'total_view_count' : bigint,
  'created_by_display_name' : [] | [string],
  'created_at' : SystemTime,
  'created_by_unique_user_name' : [] | [string],
  'video_uid' : string,
  'created_by_user_principal_id' : Principal,
  'hot_or_not_feed_ranking_score' : [] | [bigint],
  'liked_by_me' : boolean,
  'created_by_profile_photo_url' : [] | [string],
}
export interface PostDetailsFromFrontend {
  'hashtags' : Array<string>,
  'description' : string,
  'video_uid' : string,
  'creator_consent_for_inclusion_in_hot_or_not' : boolean,
}
export type PostStatus = { 'BannedForExplicitness' : null } |
  { 'BannedDueToUserReporting' : null } |
  { 'Uploaded' : null } |
  { 'CheckingExplicitness' : null } |
  { 'ReadyToView' : null } |
  { 'Transcoding' : null } |
  { 'Deleted' : null };
export type PostViewDetailsFromFrontend = {
    'WatchedMultipleTimes' : {
      'percentage_watched' : number,
      'watch_count' : number,
    }
  } |
  { 'WatchedPartially' : { 'percentage_watched' : number } };
export type Result = { 'Ok' : Array<PostDetailsForFrontend> } |
  { 'Err' : GetPostsOfUserProfileError };
export type Result_1 = { 'Ok' : Array<Principal> } |
  { 'Err' : GetFollowerOrFollowingError };
export type Result_2 = { 'Ok' : Array<[bigint, TokenEventV1]> } |
  { 'Err' : GetFollowerOrFollowingError };
export type Result_3 = { 'Ok' : boolean } |
  { 'Err' : FollowAnotherUserProfileError };
export type Result_4 = { 'Ok' : boolean } |
  { 'Err' : AnotherUserFollowedMeError };
export type Result_5 = { 'Ok' : UserProfileDetailsForFrontend } |
  { 'Err' : UpdateProfileDetailsError };
export type Result_6 = { 'Ok' : null } |
  { 'Err' : UpdateProfileSetUniqueUsernameError };
export interface SystemTime {
  'nanos_since_epoch' : number,
  'secs_since_epoch' : bigint,
}
export type TokenEventV1 = { 'Stake' : null } |
  { 'Burn' : null } |
  { 'Mint' : { 'timestamp' : SystemTime, 'details' : MintEvent } } |
  { 'Transfer' : null };
export type UpdateProfileDetailsError = { 'NotAuthorized' : null };
export type UpdateProfileSetUniqueUsernameError = {
    'UsernameAlreadyTaken' : null
  } |
  { 'UserIndexCrossCanisterCallFailed' : null } |
  { 'SendingCanisterDoesNotMatchUserCanisterId' : null } |
  { 'NotAuthorized' : null } |
  { 'UserCanisterEntryDoesNotExist' : null };
export interface UserProfileDetailsForFrontend {
  'unique_user_name' : [] | [string],
  'following_count' : bigint,
  'profile_picture_url' : [] | [string],
  'display_name' : [] | [string],
  'principal_id' : Principal,
  'profile_stats' : UserProfileGlobalStats,
  'followers_count' : bigint,
}
export interface UserProfileGlobalStats {
  'lifetime_earnings' : bigint,
  'hots_earned_count' : bigint,
  'nots_earned_count' : bigint,
}
export interface UserProfileUpdateDetailsFromFrontend {
  'profile_picture_url' : [] | [string],
  'display_name' : [] | [string],
}
export interface _SERVICE {
  'add_post' : ActorMethod<[PostDetailsFromFrontend], bigint>,
  'get_following_status_do_i_follow_this_user' : ActorMethod<
    [Principal],
    boolean
  >,
  'get_individual_post_details_by_id' : ActorMethod<
    [bigint],
    PostDetailsForFrontend
  >,
  'get_posts_of_this_user_profile_with_pagination' : ActorMethod<
    [bigint, bigint],
    Result
  >,
  'get_principals_i_follow_paginated' : ActorMethod<[bigint, bigint], Result_1>,
  'get_principals_that_follow_me_paginated' : ActorMethod<
    [bigint, bigint],
    Result_1
  >,
  'get_profile_details' : ActorMethod<[], UserProfileDetailsForFrontend>,
  'get_rewarded_for_referral' : ActorMethod<[Principal, Principal], undefined>,
  'get_rewarded_for_signing_up' : ActorMethod<[], undefined>,
  'get_user_utility_token_transaction_history_with_pagination' : ActorMethod<
    [bigint, bigint],
    Result_2
  >,
  'get_utility_token_balance' : ActorMethod<[], bigint>,
  'get_well_known_principal_value' : ActorMethod<
    [KnownPrincipalType],
    [] | [Principal]
  >,
  'return_cycles_to_user_index_canister' : ActorMethod<[], undefined>,
  'update_post_add_view_details' : ActorMethod<
    [bigint, PostViewDetailsFromFrontend],
    undefined
  >,
  'update_post_as_ready_to_view' : ActorMethod<[bigint], undefined>,
  'update_post_increment_share_count' : ActorMethod<[bigint], bigint>,
  'update_post_toggle_like_status_by_caller' : ActorMethod<[bigint], boolean>,
  'update_principals_i_follow_toggle_list_with_principal_specified' : ActorMethod<
    [Principal],
    Result_3
  >,
  'update_principals_that_follow_me_toggle_list_with_specified_principal' : ActorMethod<
    [Principal],
    Result_4
  >,
  'update_profile_display_details' : ActorMethod<
    [UserProfileUpdateDetailsFromFrontend],
    Result_5
  >,
  'update_profile_set_unique_username_once' : ActorMethod<[string], Result_6>,
}
