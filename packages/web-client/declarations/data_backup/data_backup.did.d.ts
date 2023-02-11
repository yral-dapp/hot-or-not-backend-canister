import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface AllUserData {
  'user_principal_id' : Principal,
  'user_canister_id' : Principal,
  'canister_data' : UserOwnedCanisterData,
}
export interface BackupStatistics { 'number_of_user_entries' : bigint }
export interface DataBackupInitArgs {
  'known_principal_ids' : [] | [Array<[KnownPrincipalType, Principal]>],
  'access_control_map' : [] | [Array<[Principal, Array<UserAccessRole>]>],
}
export interface HotOrNotFeedDetails {
  'upvotes' : Array<Principal>,
  'score' : bigint,
  'downvotes' : Array<Principal>,
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
export interface Post {
  'id' : bigint,
  'status' : PostStatus,
  'share_count' : bigint,
  'hashtags' : Array<string>,
  'description' : string,
  'created_at' : SystemTime,
  'likes' : Array<Principal>,
  'video_uid' : string,
  'view_stats' : PostViewStatistics,
  'hot_or_not_feed_details' : [] | [HotOrNotFeedDetails],
  'homefeed_ranking_score' : bigint,
  'creator_consent_for_inclusion_in_hot_or_not' : boolean,
}
export type PostStatus = { 'BannedForExplicitness' : null } |
  { 'BannedDueToUserReporting' : null } |
  { 'Uploaded' : null } |
  { 'CheckingExplicitness' : null } |
  { 'ReadyToView' : null } |
  { 'Transcoding' : null } |
  { 'Deleted' : null };
export interface PostViewStatistics {
  'total_view_count' : bigint,
  'average_watch_percentage' : number,
  'threshold_view_count' : bigint,
}
export interface ProfileDetails {
  'profile_picture_url' : [] | [string],
  'display_name' : [] | [string],
}
export interface SystemTime {
  'nanos_since_epoch' : number,
  'secs_since_epoch' : bigint,
}
export interface TokenBalance {
  'utility_token_balance' : bigint,
  'utility_token_transaction_history_v1' : Array<[bigint, TokenEventV1]>,
  'utility_token_transaction_history' : Array<[SystemTime, TokenEvent]>,
}
export type TokenEvent = { 'Stake' : null } |
  { 'Burn' : null } |
  { 'Mint' : MintEvent } |
  { 'Transfer' : null };
export type TokenEventV1 = { 'Stake' : null } |
  { 'Burn' : null } |
  { 'Mint' : { 'timestamp' : SystemTime, 'details' : MintEvent } } |
  { 'Transfer' : null };
export type UserAccessRole = { 'CanisterController' : null } |
  { 'ProfileOwner' : null } |
  { 'CanisterAdmin' : null } |
  { 'ProjectCanister' : null };
export interface UserOwnedCanisterData {
  'unique_user_name' : string,
  'principals_i_follow' : Array<Principal>,
  'token_data' : TokenBalance,
  'all_created_posts' : Array<[bigint, Post]>,
  'profile' : ProfileDetails,
  'principals_that_follow_me' : Array<Principal>,
}
export interface _SERVICE {
  'get_current_backup_statistics' : ActorMethod<[], BackupStatistics>,
  'get_individual_users_backup_data_entry' : ActorMethod<
    [Principal],
    [] | [AllUserData]
  >,
  'get_user_roles' : ActorMethod<[Principal], Array<UserAccessRole>>,
  'get_well_known_principal_value' : ActorMethod<
    [KnownPrincipalType],
    [] | [Principal]
  >,
  'receive_all_token_transactions_from_individual_user_canister' : ActorMethod<
    [Array<[bigint, TokenEventV1]>, Principal],
    undefined
  >,
  'receive_all_user_posts_from_individual_user_canister' : ActorMethod<
    [Array<Post>, Principal],
    undefined
  >,
  'receive_current_token_balance_from_individual_user_canister' : ActorMethod<
    [bigint, Principal],
    undefined
  >,
  'receive_principals_i_follow_from_individual_user_canister' : ActorMethod<
    [Array<Principal>, Principal],
    undefined
  >,
  'receive_principals_that_follow_me_from_individual_user_canister' : ActorMethod<
    [Array<Principal>, Principal],
    undefined
  >,
  'receive_profile_details_from_individual_user_canister' : ActorMethod<
    [[] | [string], [] | [string], [] | [string], Principal, Principal],
    undefined
  >,
  'receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister' : ActorMethod<
    [Array<[string, Principal]>],
    undefined
  >,
  'receive_user_principal_id_to_canister_id_mapping_from_user_index_canister' : ActorMethod<
    [Array<[Principal, Principal]>],
    undefined
  >,
  'restore_backed_up_data_to_individual_user_canisters' : ActorMethod<
    [],
    undefined
  >,
  'send_restore_data_back_to_user_index_canister' : ActorMethod<[], undefined>,
  'update_user_add_role' : ActorMethod<[UserAccessRole, Principal], undefined>,
  'update_user_remove_role' : ActorMethod<
    [UserAccessRole, Principal],
    undefined
  >,
}
