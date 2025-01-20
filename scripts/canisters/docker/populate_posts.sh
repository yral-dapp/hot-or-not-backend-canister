#!/bin/bash

USER_CANISTER_RAW="$(dfx canister call user_index get_requester_principals_canister_id_create_if_not_exists "()")"
export USER_CANISTER="$(echo $USER_CANISTER_RAW | awk '{print $2}' | sed 's/\")//g' | sed 's/\"//g')"

echo "Populating videos to $USER_CANISTER (this might take a bit)"

add_post() {
    dfx canister call "$USER_CANISTER" add_post_v2 "(record {
    hashtags = vec { \"test123\"; \"test231\"; \"hashtag2\"; };
    description = \"Test Post 123\";
    video_uid = \"$1\";
    creator_consent_for_inclusion_in_hot_or_not = true;
    is_nsfw = false
})"
}

export -f add_post
cat scripts/canisters/docker/uid_list | parallel -j0 add_post
unset -f add_post
unset USER_CANISTER

echo "Done"

