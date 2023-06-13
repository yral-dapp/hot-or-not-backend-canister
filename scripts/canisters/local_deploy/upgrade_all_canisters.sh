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

dfx build configuration
dfx build data_backup
dfx build individual_user_template
dfx build user_index
dfx build post_cache

if [[ $skip_test != true ]] 
then
  cargo test
fi

dfx canister install configuration --mode upgrade --argument "(record {})"
dfx canister install data_backup --mode upgrade --argument "(record {})"
dfx canister install post_cache --mode upgrade --argument "(record {})"
dfx canister install user_index --mode upgrade --argument "(record {})"
dfx canister call user_index update_user_index_upgrade_user_canisters_with_latest_wasm
