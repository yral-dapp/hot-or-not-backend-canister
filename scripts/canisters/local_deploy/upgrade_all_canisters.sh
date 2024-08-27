#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf "Usage: \n[-s Skip test] \n[-h Display help] \n";
  exit 0;
}

skip_test=false

while getopts "sh" arg; do
  case $arg in
    s)
      skip_test=true
      ;;
    h)
      usage
      ;;
  esac
done

dfx build individual_user_template
gzip -f -1 ./target/wasm32-unknown-unknown/release/individual_user_template.wasm
dfx build user_index
gzip -f -1 ./target/wasm32-unknown-unknown/release/user_index.wasm
dfx build post_cache
gzip -f -1 ./target/wasm32-unknown-unknown/release/post_cache.wasm
dfx build platform_orchestrator
gzip -f -1 ./target/wasm32-unknown-unknown/release/platform_orchestrator.wasm

if [[ $skip_test != true ]]
then
  cargo test
fi

dfx canister install post_cache --mode upgrade --argument "(record {
    version= \"v1.1.0\"
})"
dfx canister install user_index --mode upgrade --argument "(record {
  version= \"v1.1.0\"
})"

scripts/canisters/local_deploy/start_upgrades.sh
