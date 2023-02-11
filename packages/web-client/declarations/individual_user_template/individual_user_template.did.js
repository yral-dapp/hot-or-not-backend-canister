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
  const IndividualUserTemplateInitArgs = IDL.Record({
    'known_principal_ids' : IDL.Opt(
      IDL.Vec(IDL.Tuple(KnownPrincipalType, IDL.Principal))
    ),
    'profile_owner' : IDL.Opt(IDL.Principal),
  });
  const PostDetailsFromFrontend = IDL.Record({
    'hashtags' : IDL.Vec(IDL.Text),
    'description' : IDL.Text,
    'video_uid' : IDL.Text,
    'creator_consent_for_inclusion_in_hot_or_not' : IDL.Bool,
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
  const SystemTime = IDL.Record({
    'nanos_since_epoch' : IDL.Nat32,
    'secs_since_epoch' : IDL.Nat64,
  });
  const PostDetailsForFrontend = IDL.Record({
    'id' : IDL.Nat64,
    'status' : PostStatus,
    'home_feed_ranking_score' : IDL.Nat64,
    'hashtags' : IDL.Vec(IDL.Text),
    'like_count' : IDL.Nat64,
    'description' : IDL.Text,
    'total_view_count' : IDL.Nat64,
    'created_by_display_name' : IDL.Opt(IDL.Text),
    'created_at' : SystemTime,
    'created_by_unique_user_name' : IDL.Opt(IDL.Text),
    'video_uid' : IDL.Text,
    'created_by_user_principal_id' : IDL.Principal,
    'hot_or_not_feed_ranking_score' : IDL.Opt(IDL.Nat64),
    'liked_by_me' : IDL.Bool,
    'created_by_profile_photo_url' : IDL.Opt(IDL.Text),
  });
  const GetPostsOfUserProfileError = IDL.Variant({
    'ReachedEndOfItemsList' : IDL.Null,
    'InvalidBoundsPassed' : IDL.Null,
    'ExceededMaxNumberOfItemsAllowedInOneRequest' : IDL.Null,
  });
  const Result = IDL.Variant({
    'Ok' : IDL.Vec(PostDetailsForFrontend),
    'Err' : GetPostsOfUserProfileError,
  });
  const GetFollowerOrFollowingError = IDL.Variant({
    'ReachedEndOfItemsList' : IDL.Null,
    'InvalidBoundsPassed' : IDL.Null,
    'ExceededMaxNumberOfItemsAllowedInOneRequest' : IDL.Null,
  });
  const Result_1 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Principal),
    'Err' : GetFollowerOrFollowingError,
  });
  const UserProfileGlobalStats = IDL.Record({
    'lifetime_earnings' : IDL.Nat64,
    'hots_earned_count' : IDL.Nat64,
    'nots_earned_count' : IDL.Nat64,
  });
  const UserProfileDetailsForFrontend = IDL.Record({
    'unique_user_name' : IDL.Opt(IDL.Text),
    'following_count' : IDL.Nat64,
    'profile_picture_url' : IDL.Opt(IDL.Text),
    'display_name' : IDL.Opt(IDL.Text),
    'principal_id' : IDL.Principal,
    'profile_stats' : UserProfileGlobalStats,
    'followers_count' : IDL.Nat64,
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
  const Result_2 = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Tuple(IDL.Nat64, TokenEventV1)),
    'Err' : GetFollowerOrFollowingError,
  });
  const PostViewDetailsFromFrontend = IDL.Variant({
    'WatchedMultipleTimes' : IDL.Record({
      'percentage_watched' : IDL.Nat8,
      'watch_count' : IDL.Nat8,
    }),
    'WatchedPartially' : IDL.Record({ 'percentage_watched' : IDL.Nat8 }),
  });
  const FollowAnotherUserProfileError = IDL.Variant({
    'UserToFollowDoesNotExist' : IDL.Null,
    'UserIndexCrossCanisterCallFailed' : IDL.Null,
    'UserITriedToFollowCrossCanisterCallFailed' : IDL.Null,
    'UsersICanFollowListIsFull' : IDL.Null,
    'MyCanisterIDDoesNotMatchMyPrincipalCanisterIDMappingSeenByUserITriedToFollow' : IDL.Null,
    'UserITriedToFollowDidNotFindMe' : IDL.Null,
    'NotAuthorized' : IDL.Null,
    'UserITriedToFollowHasTheirFollowersListFull' : IDL.Null,
  });
  const Result_3 = IDL.Variant({
    'Ok' : IDL.Bool,
    'Err' : FollowAnotherUserProfileError,
  });
  const AnotherUserFollowedMeError = IDL.Variant({
    'UserIndexCrossCanisterCallFailed' : IDL.Null,
    'FollowersListFull' : IDL.Null,
    'NotAuthorized' : IDL.Null,
    'UserTryingToFollowMeDoesNotExist' : IDL.Null,
  });
  const Result_4 = IDL.Variant({
    'Ok' : IDL.Bool,
    'Err' : AnotherUserFollowedMeError,
  });
  const UserProfileUpdateDetailsFromFrontend = IDL.Record({
    'profile_picture_url' : IDL.Opt(IDL.Text),
    'display_name' : IDL.Opt(IDL.Text),
  });
  const UpdateProfileDetailsError = IDL.Variant({ 'NotAuthorized' : IDL.Null });
  const Result_5 = IDL.Variant({
    'Ok' : UserProfileDetailsForFrontend,
    'Err' : UpdateProfileDetailsError,
  });
  const UpdateProfileSetUniqueUsernameError = IDL.Variant({
    'UsernameAlreadyTaken' : IDL.Null,
    'UserIndexCrossCanisterCallFailed' : IDL.Null,
    'SendingCanisterDoesNotMatchUserCanisterId' : IDL.Null,
    'NotAuthorized' : IDL.Null,
    'UserCanisterEntryDoesNotExist' : IDL.Null,
  });
  const Result_6 = IDL.Variant({
    'Ok' : IDL.Null,
    'Err' : UpdateProfileSetUniqueUsernameError,
  });
  return IDL.Service({
    'add_post' : IDL.Func([PostDetailsFromFrontend], [IDL.Nat64], []),
    'get_following_status_do_i_follow_this_user' : IDL.Func(
        [IDL.Principal],
        [IDL.Bool],
        ['query'],
      ),
    'get_individual_post_details_by_id' : IDL.Func(
        [IDL.Nat64],
        [PostDetailsForFrontend],
        ['query'],
      ),
    'get_posts_of_this_user_profile_with_pagination' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [Result],
        ['query'],
      ),
    'get_principals_i_follow_paginated' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [Result_1],
        ['query'],
      ),
    'get_principals_that_follow_me_paginated' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [Result_1],
        ['query'],
      ),
    'get_profile_details' : IDL.Func(
        [],
        [UserProfileDetailsForFrontend],
        ['query'],
      ),
    'get_rewarded_for_referral' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [],
        [],
      ),
    'get_rewarded_for_signing_up' : IDL.Func([], [], []),
    'get_user_utility_token_transaction_history_with_pagination' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [Result_2],
        ['query'],
      ),
    'get_utility_token_balance' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_well_known_principal_value' : IDL.Func(
        [KnownPrincipalType],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'return_cycles_to_user_index_canister' : IDL.Func([], [], []),
    'update_post_add_view_details' : IDL.Func(
        [IDL.Nat64, PostViewDetailsFromFrontend],
        [],
        [],
      ),
    'update_post_as_ready_to_view' : IDL.Func([IDL.Nat64], [], []),
    'update_post_increment_share_count' : IDL.Func(
        [IDL.Nat64],
        [IDL.Nat64],
        [],
      ),
    'update_post_toggle_like_status_by_caller' : IDL.Func(
        [IDL.Nat64],
        [IDL.Bool],
        [],
      ),
    'update_principals_i_follow_toggle_list_with_principal_specified' : IDL.Func(
        [IDL.Principal],
        [Result_3],
        [],
      ),
    'update_principals_that_follow_me_toggle_list_with_specified_principal' : IDL.Func(
        [IDL.Principal],
        [Result_4],
        [],
      ),
    'update_profile_display_details' : IDL.Func(
        [UserProfileUpdateDetailsFromFrontend],
        [Result_5],
        [],
      ),
    'update_profile_set_unique_username_once' : IDL.Func(
        [IDL.Text],
        [Result_6],
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
  const IndividualUserTemplateInitArgs = IDL.Record({
    'known_principal_ids' : IDL.Opt(
      IDL.Vec(IDL.Tuple(KnownPrincipalType, IDL.Principal))
    ),
    'profile_owner' : IDL.Opt(IDL.Principal),
  });
  return [IndividualUserTemplateInitArgs];
};
