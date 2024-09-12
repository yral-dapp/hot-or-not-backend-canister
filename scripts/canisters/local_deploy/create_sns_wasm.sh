#!/usr/bin/env bash

dfx canister create --no-wallet sns_wasm
SNS_WASM_CANISTER="$(dfx canister id sns_wasm)"
dfx canister install sns_wasm --wasm wasms/sns-wasm-canister.wasm --argument "(record {
    allowed_principals = vec {};
    access_controls_enabled = false;
    sns_subnet_ids = vec {};
})"

add_wasm() {
    file_hash=$(sha256sum $1 | cut -d ' ' -f 1)
    char=$(hexdump -ve '1/1 "%.2x"' "$1")
    char_escaped=$(printf "%s" "$char" | sed 's/../\\&/g')
    file_hash_escaped=$(printf "%s" "$file_hash" | sed 's/../\\&/g')
    printf "(record {
        hash = blob \"%s\";
        wasm = opt record {
            wasm = blob \"%s\";
            canister_type = %d;
        };
    })" "$file_hash_escaped" "$char_escaped" "$2" > argument


    dfx canister call "$SNS_WASM_CANISTER" add_wasm --argument-file argument
}

add_wasm wasms/root.wasm.gz 1
add_wasm wasms/governance.wasm.gz 2
add_wasm wasms/ledger.wasm.gz 3
add_wasm wasms/swap.wasm.gz 4
add_wasm wasms/archive.wasm.gz 5
add_wasm wasms/index.wasm.gz 6

