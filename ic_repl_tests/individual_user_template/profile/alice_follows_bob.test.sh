

import user_index_canister = "renrk-eyaaa-aaaaa-aaada-cai";

identity bob "~/.config/dfx/identity/bob/identity.pem";

let bob_canister = call user_index_canister.get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(null);

let response = call bob_canister.get_profile_details();

identity alice "~/.config/dfx/identity/alice/identity.pem";

let alice_canister = call user_index_canister.get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(null);

call alice_canister.update_principals_i_follow_toggle_list_with_principal_specified(response.principal_id);