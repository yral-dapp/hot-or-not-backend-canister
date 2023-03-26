import user_index_canister = "rkp4c-7iaaa-aaaaa-aaaca-cai";

identity default;

let my_canister = call user_index_canister.get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(null);

let result = call my_canister.add_post_v2(record { hashtags = vec { "a"; "b"; "c" }; description = "This is post from integration test"; video_uid = "#1234567890"; creator_consent_for_inclusion_in_hot_or_not = false; });
let post_id = result.Ok;

call my_canister.update_post_toggle_like_status_by_caller(post_id);
call my_canister.get_individual_post_details_by_id(post_id);

call my_canister.update_post_toggle_like_status_by_caller(post_id);
call my_canister.get_individual_post_details_by_id(post_id);

call my_canister.update_post_toggle_like_status_by_caller(post_id);
call my_canister.get_individual_post_details_by_id(post_id);
