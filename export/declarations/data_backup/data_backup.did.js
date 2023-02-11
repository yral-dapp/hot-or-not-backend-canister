export const idlFactory = ({ IDL }) => {
  const KnownPrincipalType = IDL.Variant({
    'CanisterIdUserIndex' : IDL.Null,
    'CanisterIdConfiguration' : IDL.Null,
    'CanisterIdProjectMemberIndex' : IDL.Null,
    'CanisterIdTopicCacheIndex' : IDL.Null,
    'CanisterIdRootCanister' : IDL.Null,
    'CanisterIdDataBackup' : IDL.Null,
    'CanisterIdPostCache' : IDL.Null,
    'CanisterIdSNSController' : IDL.Null,
    'UserIdGlobalSuperAdmin' : IDL.Null,
  });
  const UserAccessRole = IDL.Variant({
    'CanisterController' : IDL.Null,
    'ProfileOwner' : IDL.Null,
    'CanisterAdmin' : IDL.Null,
    'ProjectCanister' : IDL.Null,
  });
  const DataBackupInitArgs = IDL.Record({
    'known_principal_ids' : IDL.Opt(
      IDL.Vec(IDL.Tuple(KnownPrincipalType, IDL.Principal))
    ),
    'access_control_map' : IDL.Opt(
      IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Vec(UserAccessRole)))
    ),
  });
  const BackupStatistics = IDL.Record({ 'number_of_user_entries' : IDL.Nat64 });
  const SystemTime = IDL.Record({
    'nanos_since_epoch' : IDL.Nat32,
    'secs_since_epoch' : IDL.Nat64,
  });
  const MintEvent = IDL.Variant({
    'NewUserSignup' : IDL.Record({ 'new_user_principal_id' : IDL.Principal }),
    'Referral' : IDL.Record({
      'referrer_user_principal_id' : IDL.Principal,
      'referee_user_principal_id' : IDL.Principal,
    }),
  });
  const TokenEventV1 = IDL.Variant({
    'Stake' : IDL.Null,
    'Burn' : IDL.Null,
    'Mint' : IDL.Record({ 'timestamp' : SystemTime, 'details' : MintEvent }),
    'Transfer' : IDL.Null,
  });
  const TokenEvent = IDL.Variant({
    'Stake' : IDL.Null,
    'Burn' : IDL.Null,
    'Mint' : MintEvent,
    'Transfer' : IDL.Null,
  });
  const TokenBalance = IDL.Record({
    'utility_token_balance' : IDL.Nat64,
    'utility_token_transaction_history_v1' : IDL.Vec(
      IDL.Tuple(IDL.Nat64, TokenEventV1)
    ),
    'utility_token_transaction_history' : IDL.Vec(
      IDL.Tuple(SystemTime, TokenEvent)
    ),
  });
  const PostStatus = IDL.Variant({
    'BannedForExplicitness' : IDL.Null,
    'BannedDueToUserReporting' : IDL.Null,
    'Uploaded' : IDL.Null,
    'CheckingExplicitness' : IDL.Null,
    'ReadyToView' : IDL.Null,
    'Transcoding' : IDL.Null,
    'Deleted' : IDL.Null,
  });
  const PostViewStatistics = IDL.Record({
    'total_view_count' : IDL.Nat64,
    'average_watch_percentage' : IDL.Nat8,
    'threshold_view_count' : IDL.Nat64,
  });
  const HotOrNotFeedDetails = IDL.Record({
    'upvotes' : IDL.Vec(IDL.Principal),
    'score' : IDL.Nat64,
    'downvotes' : IDL.Vec(IDL.Principal),
  });
  const Post = IDL.Record({
    'id' : IDL.Nat64,
    'status' : PostStatus,
    'share_count' : IDL.Nat64,
    'hashtags' : IDL.Vec(IDL.Text),
    'description' : IDL.Text,
    'created_at' : SystemTime,
    'likes' : IDL.Vec(IDL.Principal),
    'video_uid' : IDL.Text,
    'view_stats' : PostViewStatistics,
    'hot_or_not_feed_details' : IDL.Opt(HotOrNotFeedDetails),
    'homefeed_ranking_score' : IDL.Nat64,
    'creator_consent_for_inclusion_in_hot_or_not' : IDL.Bool,
  });
  const ProfileDetails = IDL.Record({
    'profile_picture_url' : IDL.Opt(IDL.Text),
    'display_name' : IDL.Opt(IDL.Text),
  });
  const UserOwnedCanisterData = IDL.Record({
    'unique_user_name' : IDL.Text,
    'principals_i_follow' : IDL.Vec(IDL.Principal),
    'token_data' : TokenBalance,
    'all_created_posts' : IDL.Vec(IDL.Tuple(IDL.Nat64, Post)),
    'profile' : ProfileDetails,
    'principals_that_follow_me' : IDL.Vec(IDL.Principal),
  });
  const AllUserData = IDL.Record({
    'user_principal_id' : IDL.Principal,
    'user_canister_id' : IDL.Principal,
    'canister_data' : UserOwnedCanisterData,
  });
  return IDL.Service({
    'get_current_backup_statistics' : IDL.Func(
        [],
        [BackupStatistics],
        ['query'],
      ),
    'get_individual_users_backup_data_entry' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(AllUserData)],
        ['query'],
      ),
    'get_user_roles' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(UserAccessRole)],
        ['query'],
      ),
    'get_well_known_principal_value' : IDL.Func(
        [KnownPrincipalType],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'receive_all_token_transactions_from_individual_user_canister' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Nat64, TokenEventV1)), IDL.Principal],
        [],
        [],
      ),
    'receive_all_user_posts_from_individual_user_canister' : IDL.Func(
        [IDL.Vec(Post), IDL.Principal],
        [],
        [],
      ),
    'receive_current_token_balance_from_individual_user_canister' : IDL.Func(
        [IDL.Nat64, IDL.Principal],
        [],
        [],
      ),
    'receive_principals_i_follow_from_individual_user_canister' : IDL.Func(
        [IDL.Vec(IDL.Principal), IDL.Principal],
        [],
        [],
      ),
    'receive_principals_that_follow_me_from_individual_user_canister' : IDL.Func(
        [IDL.Vec(IDL.Principal), IDL.Principal],
        [],
        [],
      ),
    'receive_profile_details_from_individual_user_canister' : IDL.Func(
        [
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Principal,
          IDL.Principal,
        ],
        [],
        [],
      ),
    'receive_unique_user_name_to_user_principal_id_mapping_from_user_index_canister' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Principal))],
        [],
        [],
      ),
    'receive_user_principal_id_to_canister_id_mapping_from_user_index_canister' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Principal))],
        [],
        [],
      ),
    'restore_backed_up_data_to_individual_user_canisters' : IDL.Func(
        [],
        [],
        [],
      ),
    'send_restore_data_back_to_user_index_canister' : IDL.Func([], [], []),
    'update_user_add_role' : IDL.Func([UserAccessRole, IDL.Principal], [], []),
    'update_user_remove_role' : IDL.Func(
        [UserAccessRole, IDL.Principal],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const KnownPrincipalType = IDL.Variant({
    'CanisterIdUserIndex' : IDL.Null,
    'CanisterIdConfiguration' : IDL.Null,
    'CanisterIdProjectMemberIndex' : IDL.Null,
    'CanisterIdTopicCacheIndex' : IDL.Null,
    'CanisterIdRootCanister' : IDL.Null,
    'CanisterIdDataBackup' : IDL.Null,
    'CanisterIdPostCache' : IDL.Null,
    'CanisterIdSNSController' : IDL.Null,
    'UserIdGlobalSuperAdmin' : IDL.Null,
  });
  const UserAccessRole = IDL.Variant({
    'CanisterController' : IDL.Null,
    'ProfileOwner' : IDL.Null,
    'CanisterAdmin' : IDL.Null,
    'ProjectCanister' : IDL.Null,
  });
  const DataBackupInitArgs = IDL.Record({
    'known_principal_ids' : IDL.Opt(
      IDL.Vec(IDL.Tuple(KnownPrincipalType, IDL.Principal))
    ),
    'access_control_map' : IDL.Opt(
      IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Vec(UserAccessRole)))
    ),
  });
  return [DataBackupInitArgs];
};
