#!/usr/bin/env bash

DFX_IC_COMMIT=94bbea43c7585a1ef970bd569a447c269af9650b

echo -e "\n\n\n\n\n\n\n\n\n"

DIR=/app/target/ic

# if [ ! -d "$DIR" ]; then
#   mkdir -p "$DIR"
# fi

curl -o ./icp_index.wasm.gz "https://download.dfinity.systems/ic/$DFX_IC_COMMIT/canisters/ic-icp-index-canister.wasm.gz"

curl -o ./icp_ledger.wasm.gz "https://download.dfinity.systems/ic/$DFX_IC_COMMIT/canisters/ledger-canister.wasm.gz"
pwd && ls -la /app/target/ic

dfx identity use minter
MINTER_ACCOUNT_ID=$(dfx ledger account-id)

dfx identity use admin
LEDGER_ACCOUNT_ID=$(dfx ledger account-id)

dfx deploy --specified-id ryjl3-tyaaa-aaaaa-aaaba-cai icp_ledger --argument "
  (variant {
    Init = record {
      minting_account = \"$MINTER_ACCOUNT_ID\";
      initial_values = vec {
        record {
          \"$LEDGER_ACCOUNT_ID\";
          record {
            e8s = 100_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10_000 : nat64;
      };
      token_symbol = opt \"ICP\";
      token_name = opt \"Local ICP\";
    }
  })
"

LEDGER_ACCOUNT_ID=$(dfx canister id icp_ledger)

dfx deploy icp_index --specified-id qhbym-qaaaa-aaaaa-aaafq-cai --argument '(record {ledger_id = principal"'${LEDGER_ACCOUNT_ID}'";})'

dfx canister call qhbym-qaaaa-aaaaa-aaafq-cai ledger_id '()'

dfx canister call qhbym-qaaaa-aaaaa-aaafq-cai status '()'