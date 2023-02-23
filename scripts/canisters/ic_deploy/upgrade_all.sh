#!/usr/bin/env bash
set -euo pipefail

dfx build --network=ic configuration
gzip -f -1 ./target/wasm32-unknown-unknown/release/configuration.wasm
dfx build --network=ic data_backup
gzip -f -1 ./target/wasm32-unknown-unknown/release/data_backup.wasm
dfx build --network=ic individual_user_template
gzip -f -1 ./target/wasm32-unknown-unknown/release/individual_user_template.wasm
dfx build --network=ic user_index
gzip -f -1 ./target/wasm32-unknown-unknown/release/user_index.wasm
dfx build --network=ic post_cache
gzip -f -1 ./target/wasm32-unknown-unknown/release/post_cache.wasm

# dfx canister install configuration --network ic --mode upgrade --argument "(record {})"
# dfx canister install data_backup --network ic --mode upgrade --argument "(record {})"
# dfx canister install post_cache --network ic --mode upgrade --argument "(record {})"
dfx canister install user_index --network ic --mode upgrade --argument "(record {})"
dfx canister call user_index update_user_index_upgrade_user_canisters_with_latest_wasm --network ic --async