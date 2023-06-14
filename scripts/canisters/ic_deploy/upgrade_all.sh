#!/usr/bin/env bash
set -euo pipefail

dfx build --network=ic individual_user_template
gzip -f -1 ./target/wasm32-unknown-unknown/release/individual_user_template.wasm
dfx build --network=ic user_index
dfx build --network=ic configuration
dfx build --network=ic data_backup
dfx build --network=ic post_cache

dfx canister install configuration --network ic --mode upgrade --argument "(record {})"
dfx canister install data_backup --network ic --mode upgrade --argument "(record {})"
dfx canister install post_cache --network ic --mode upgrade --argument "(record {})"
dfx canister install user_index --network ic --mode upgrade --argument "(record {})"
dfx canister call user_index update_user_index_upgrade_user_canisters_with_latest_wasm --network ic --async