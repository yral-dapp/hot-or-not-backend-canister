import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface ConfigurationInitArgs {
  'known_principal_ids' : [] | [Array<[KnownPrincipalType, Principal]>],
  'signups_enabled' : [] | [boolean],
  'access_control_map' : [] | [Array<[Principal, Array<UserAccessRole>]>],
}
export type ErrorUpdateListOfWellKnownPrincipals = { 'Unauthorized' : null };
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
  { 'Err' : ErrorUpdateListOfWellKnownPrincipals };
export type UserAccessRole = { 'CanisterController' : null } |
  { 'ProfileOwner' : null } |
  { 'CanisterAdmin' : null } |
  { 'ProjectCanister' : null };
export interface _SERVICE {
  'are_signups_enabled' : ActorMethod<[], boolean>,
  'get_current_list_of_all_well_known_principal_values' : ActorMethod<
    [],
    Array<[KnownPrincipalType, Principal]>
  >,
  'get_user_roles' : ActorMethod<[Principal], Array<UserAccessRole>>,
  'get_well_known_principal_value' : ActorMethod<
    [KnownPrincipalType],
    [] | [Principal]
  >,
  'toggle_signups_enabled' : ActorMethod<[], undefined>,
  'update_list_of_well_known_principals' : ActorMethod<
    [KnownPrincipalType, Principal],
    Result
  >,
  'update_user_add_role' : ActorMethod<[UserAccessRole, Principal], undefined>,
  'update_user_remove_role' : ActorMethod<
    [UserAccessRole, Principal],
    undefined
  >,
}
