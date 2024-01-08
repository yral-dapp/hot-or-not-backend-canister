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
dfx build configuration
dfx build data_backup
dfx build user_index
dfx build post_cache

if [[ $skip_test != true ]] 
then
  cargo test
fi

dfx canister install configuration --mode upgrade --argument "(record {})"
dfx canister install data_backup --mode upgrade --argument "(record {})"
dfx canister install post_cache --mode upgrade --argument "(record {})"
dfx canister install user_index --mode upgrade --argument "(record {
  version= \"v1.1.0\"
})"
