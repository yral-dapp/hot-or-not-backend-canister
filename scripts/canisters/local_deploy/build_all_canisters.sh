#!/usr/bin/env bash
set -euo pipefail

export CANISTER_ID_configuration=$(dfx canister id configuration)
export CANISTER_ID_data_backup=$(dfx canister id data_backup)
export CANISTER_ID_post_cache=$(dfx canister id post_cache)
export CANISTER_ID_user_index=$(dfx canister id user_index)

export USER_ID_global_super_admin=$(dfx identity get-principal)

export LOCAL_TOP_POSTS_SYNC_INTERVAL="10000000000"

dfx build configuration
gzip -f -1 ./target/wasm32-unknown-unknown/release/configuration.wasm
dfx build data_backup
gzip -f -1 ./target/wasm32-unknown-unknown/release/data_backup.wasm
dfx build individual_user_template
gzip -f -1 ./target/wasm32-unknown-unknown/release/individual_user_template.wasm
dfx build user_index
gzip -f -1 ./target/wasm32-unknown-unknown/release/user_index.wasm
dfx build post_cache
gzip -f -1 ./target/wasm32-unknown-unknown/release/post_cache.wasm