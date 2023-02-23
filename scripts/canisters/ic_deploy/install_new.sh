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

# dfx canister install data_backup --mode reinstall --network ic --argument "(record {
#   known_principal_ids = opt vec {
#     record {
#       variant { CanisterIdConfiguration };
#       principal \"$(dfx canister id configuration --network ic)\";
#     };
#   };
#   access_control_map = opt vec {
#     record {
#       principal \"$(dfx identity get-principal --network ic)\";
#       vec { variant { CanisterAdmin }; variant { CanisterController }; }
#     };
#   };
# })"

dfx canister install data_backup --network ic --mode upgrade --argument "(record { })"