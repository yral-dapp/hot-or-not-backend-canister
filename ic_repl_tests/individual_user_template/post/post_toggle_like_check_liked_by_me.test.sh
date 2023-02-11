import user_index_canister = "renrk-eyaaa-aaaaa-aaada-cai";

identity default "~/.config/dfx/identity/default/identity.pem";

let my_canister = call user_index_canister.get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(null);

let post_id = call my_canister.add_post(record { hashtags = vec { "a"; "b"; "c" }; description = "This is post from integration test"; video_uid = "#1234567890"; creator_consent_for_inclusion_in_hot_or_not = false; });

call my_canister.update_post_toggle_like_status_by_caller(post_id);
call my_canister.get_individual_post_details_by_id(post_id);

call my_canister.update_post_toggle_like_status_by_caller(post_id);
call my_canister.get_individual_post_details_by_id(post_id);

call my_canister.update_post_toggle_like_status_by_caller(post_id);
call my_canister.get_individual_post_details_by_id(post_id);
