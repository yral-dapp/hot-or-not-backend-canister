#!/usr/bin/env bash
set -euo pipefail

export USER_ID_global_super_admin=$(dfx identity get-principal)

dfx canister create --no-wallet configuration
dfx canister create --no-wallet data_backup
dfx canister create --no-wallet individual_user_template
dfx canister create --no-wallet post_cache
dfx canister create --no-wallet user_index

export CANISTER_ID_configuration=$(dfx canister id configuration)
export CANISTER_ID_data_backup=$(dfx canister id data_backup)
export CANISTER_ID_post_cache=$(dfx canister id post_cache)
export CANISTER_ID_user_index=$(dfx canister id user_index)

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
  known_principal_ids = vec {
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
  }
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
})"
