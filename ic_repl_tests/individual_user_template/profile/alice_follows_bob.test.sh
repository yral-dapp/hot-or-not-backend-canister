

import user_index_canister = "be2us-64aaa-aaaaa-qaabq-cai";


identity bob;

let bob_canister = call user_index_canister.get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(null);

let response = call bob_canister.get_profile_details();

identity alice;

let alice_canister = call user_index_canister.get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(null);

call alice_canister.update_profiles_i_follow_toggle_list_with_specified_profile(record { 
    followee_principal_id = bob;
    followee_canister_id = bob_canister;
});