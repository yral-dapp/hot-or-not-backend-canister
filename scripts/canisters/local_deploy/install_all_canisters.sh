#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf "Usage: \n[-s Skip test] \n[-h Display help] \n";
  exit 0;
}

skip_test=false

while getopts "sih" arg; do
  case $arg in
    s)
      skip_test=true
      ;;
    h)
      usage
      ;;
  esac
done

scripts/canisters/local_deploy/setup_icp_ledger.sh
scripts/canisters/local_deploy/create_sns_wasm.sh

dfx canister create --no-wallet individual_user_template
dfx canister create --no-wallet post_cache
dfx canister create --no-wallet user_index
dfx canister create --no-wallet platform_orchestrator

gzip_canister() {
  gzip -f -1 ./target/wasm32-unknown-unknown/release/$1.wasm
}

scripts/candid_generator.sh

gzip_canister individual_user_template
gzip_canister user_index
gzip_canister post_cache
gzip_canister platform_orchestrator

if [[ $skip_test != true ]]
then
  cargo test
fi


dfx canister install platform_orchestrator --argument "(record {
  version = \"v1.0.0\"
})"

dfx canister install post_cache --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
    record {
      variant { CanisterIdSnsWasm };
      principal \"$(dfx canister id sns_wasm)\";
    };
  };
  version= \"v1.0.0\"
})"

dfx canister install user_index --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { CanisterIdPlatformOrchestrator };
      principal \"$(dfx canister id platform_orchestrator)\";
    };
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
    record {
      variant { CanisterIdSnsWasm };
      principal \"$(dfx canister id sns_wasm)\";
    };
  };
  access_control_map = opt vec {
    record {
      principal \"$(dfx identity get-principal)\";
      vec { variant { CanisterAdmin }; variant { CanisterController }; }
    };
  };
  proof_of_participation = opt record {
    chain = vec {};
  };
  version= \"v1.0.0\"
})"
scripts/canisters/local_deploy/create_pool_of_individual_canister_user_index.sh

 dfx canister update-settings user_index --set-controller $(dfx canister id platform_orchestrator) --yes 
 dfx canister call platform_orchestrator register_new_subnet_orchestrator  "(principal \"$(dfx canister id user_index)\", true)"


