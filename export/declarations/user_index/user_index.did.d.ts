import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
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
export type Result = { 'Ok' : null } |
  { 'Err' : SetUniqueUsernameError };
export type SetUniqueUsernameError = { 'UsernameAlreadyTaken' : null } |
  { 'SendingCanisterDoesNotMatchUserCanisterId' : null } |
  { 'UserCanisterEntryDoesNotExist' : null };
export interface SystemTime {
  'nanos_since_epoch' : number,
  'secs_since_epoch' : bigint,
}
export interface UpgradeStatusV1 {
  'version_number' : bigint,
  'last_run_on' : SystemTime,
  'failed_canister_ids' : Array<[Principal, Principal]>,
  'successful_upgrade_count' : number,
}
export type UserAccessRole = { 'CanisterController' : null } |
  { 'ProfileOwner' : null } |
  { 'CanisterAdmin' : null } |
  { 'ProjectCanister' : null };
export interface UserIndexInitArgs {
  'known_principal_ids' : [] | [Array<[KnownPrincipalType, Principal]>],
  'access_control_map' : [] | [Array<[Principal, Array<UserAccessRole>]>],
}
export interface _SERVICE {
  'backup_all_individual_user_canisters' : ActorMethod<[], undefined>,
  'backup_data_to_backup_canister' : ActorMethod<[], undefined>,
  'get_canister_status_from_management_canister' : ActorMethod<
    [Principal],
    CanisterStatusResponse
  >,
  'get_index_details_is_user_name_taken' : ActorMethod<[string], boolean>,
  'get_index_details_last_upgrade_status' : ActorMethod<[], UpgradeStatusV1>,
  'get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer' : ActorMethod<
    [[] | [Principal]],
    Principal
  >,
  'get_user_canister_id_from_unique_user_name' : ActorMethod<
    [string],
    [] | [Principal]
  >,
  'get_user_canister_id_from_user_principal_id' : ActorMethod<
    [Principal],
    [] | [Principal]
  >,
  'get_user_roles' : ActorMethod<[Principal], Array<UserAccessRole>>,
  'get_well_known_principal_value' : ActorMethod<
    [KnownPrincipalType],
    [] | [Principal]
  >,
  'receive_data_from_backup_canister_and_restore_data_to_heap' : ActorMethod<
    [Principal, Principal, string],
    undefined
  >,
  'topup_canisters_that_need_it' : ActorMethod<[], undefined>,
  'update_index_with_unique_user_name_corresponding_to_user_principal_id' : ActorMethod<
    [string, Principal],
    Result
  >,
  'update_user_add_role' : ActorMethod<[UserAccessRole, Principal], undefined>,
  'update_user_index_upgrade_user_canisters_with_latest_wasm' : ActorMethod<
    [],
    undefined
  >,
  'update_user_remove_role' : ActorMethod<
    [UserAccessRole, Principal],
    undefined
  >,
}
