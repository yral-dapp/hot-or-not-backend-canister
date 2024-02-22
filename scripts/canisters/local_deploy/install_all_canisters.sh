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

dfx canister create --no-wallet configuration
dfx canister create --no-wallet data_backup
dfx canister create --no-wallet individual_user_template
dfx canister create --no-wallet post_cache
dfx canister create --no-wallet user_index
dfx canister create --no-wallet platform_orchestrator

dfx build individual_user_template
gzip -f -1 ./target/wasm32-unknown-unknown/release/individual_user_template.wasm
dfx build configuration
gzip -f -1 ./target/wasm32-unknown-unknown/release/configuration.wasm
dfx build data_backup
gzip -f -1 ./target/wasm32-unknown-unknown/release/data_backup.wasm
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

dfx canister install configuration --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdConfiguration };
      principal \"$(dfx canister id configuration)\";
    };
    record {
      variant { CanisterIdDataBackup };
      principal \"$(dfx canister id data_backup)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
  };
  signups_enabled = opt true;
  access_control_map = opt vec {
    record {
      principal \"$(dfx identity get-principal)\";
      vec { variant { CanisterAdmin }; variant { CanisterController }; }
    };
  };
})"

dfx canister install data_backup --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdConfiguration };
      principal \"$(dfx canister id configuration)\";
    };
    record {
      variant { CanisterIdDataBackup };
      principal \"$(dfx canister id data_backup)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
  };
  access_control_map = opt vec {
    record {
      principal \"$(dfx identity get-principal)\";
      vec { variant { CanisterAdmin }; variant { CanisterController }; }
    };
  };
})"

dfx canister install post_cache --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdConfiguration };
      principal \"$(dfx canister id configuration)\";
    };
    record {
      variant { CanisterIdDataBackup };
      principal \"$(dfx canister id data_backup)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
  };
  version= \"v1.0.0\"
})"

dfx canister install user_index --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdConfiguration };
      principal \"$(dfx canister id configuration)\";
    };
    record {
      variant { CanisterIdDataBackup };
      principal \"$(dfx canister id data_backup)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
  };
  access_control_map = opt vec {
    record {
      principal \"$(dfx identity get-principal)\";
      vec { variant { CanisterAdmin }; variant { CanisterController }; }
    };
  };
  version= \"v1.0.0\"
})"

scripts/canisters/local_deploy/create_pool_of_individual_canister_user_index.sh