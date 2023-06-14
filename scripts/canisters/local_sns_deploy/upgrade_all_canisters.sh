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
dfx build configuration
dfx build data_backup
dfx build post_cache

if [[ $skip_test != true ]] 
then
  cargo test
fi

source ./sns-testing/constants.sh normal

cd ..

quill sns \
  --canister-ids-file "./sns-testing/sns_canister_ids.json"  \
  --pem-file "${PEM_FILE}"  \
  make-upgrade-canister-proposal \
  --summary "This proposal upgrades test canister"  \
  --title "Upgrade configuration canister"  \
  --url "https://hotornot.wtf/"  \
  --target-canister-id $(dfx canister id configuration) \
  --wasm-path ./.dfx/local/canisters/configuration/configuration.wasm.gz \
  --canister-upgrade-arg "(record {})" \
"$(./sns-testing/developer_neuron_id.sh)" > msg.json
quill --insecure-local-dev-mode send --yes msg.json | grep -v "new_canister_wasm"

quill sns \
  --canister-ids-file "./sns-testing/sns_canister_ids.json"  \
  --pem-file "${PEM_FILE}"  \
  make-upgrade-canister-proposal \
  --summary "This proposal upgrades data_backup canister"  \
  --title "Upgrade data_backup canister"  \
  --url "https://hotornot.wtf/"  \
  --target-canister-id $(dfx canister id data_backup) \
  --wasm-path ./.dfx/local/canisters/data_backup/data_backup.wasm.gz \
  --canister-upgrade-arg "(record {})" \
"$(./sns-testing/developer_neuron_id.sh)" > msg.json
quill --insecure-local-dev-mode send --yes msg.json | grep -v "new_canister_wasm"

quill sns \
  --canister-ids-file "./sns-testing/sns_canister_ids.json"  \
  --pem-file "${PEM_FILE}"  \
  make-upgrade-canister-proposal \
  --summary "This proposal upgrades post_cache canister"  \
  --title "Upgrade post_cache canister"  \
  --url "https://hotornot.wtf/"  \
  --target-canister-id $(dfx canister id post_cache) \
  --wasm-path ./.dfx/local/canisters/post_cache/post_cache.wasm.gz \
  --canister-upgrade-arg "(record {})" \
"$(./sns-testing/developer_neuron_id.sh)" > msg.json
quill --insecure-local-dev-mode send --yes msg.json | grep -v "new_canister_wasm"

quill sns \
  --canister-ids-file "./sns-testing/sns_canister_ids.json"  \
  --pem-file "${PEM_FILE}"  \
  make-upgrade-canister-proposal \
  --summary "This proposal upgrades user_index canister"  \
  --title "Upgrade user_index canister"  \
  --url "https://hotornot.wtf/"  \
  --target-canister-id $(dfx canister id user_index) \
  --wasm-path ./.dfx/local/canisters/user_index/user_index.wasm.gz \
  --canister-upgrade-arg "(record {})" \
"$(./sns-testing/developer_neuron_id.sh)" > msg.json
quill --insecure-local-dev-mode send --yes msg.json | grep -v "new_canister_wasm"

