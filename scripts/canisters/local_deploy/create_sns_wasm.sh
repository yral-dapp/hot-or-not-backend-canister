#!/usr/bin/env bash

DFX_IC_COMMIT=a0207146be211cdff83321c99e9e70baa62733c7

if [ ! -e "./sns-wasm-canister.wasm" ]; then
    wget "https://download.dfinity.systems/ic/$DFX_IC_COMMIT/canisters/sns-wasm-canister.wasm.gz"
    gzip -d sns-wasm-canister.wasm.gz
fi

if [ ! -e "./sns-root-canister.wasm.gz" ]; then
    dfx sns download
    gzip -f sns-root-canister.wasm
    gzip -f sns-governance-canister.wasm
    gzip -f ic-icrc1-ledger.wasm
    gzip -f sns-swap-canister.wasm
    gzip -f ic-icrc1-archive.wasm
    gzip -f ic-icrc1-index.wasm
fi

dfx canister create --no-wallet sns_wasm
SNS_WASM_CANISTER="$(dfx canister id sns_wasm)"
dfx canister install sns_wasm --wasm ./sns-wasm-canister.wasm --argument "(record {
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

add_wasm sns-root-canister.wasm.gz 1
add_wasm sns-governance-canister.wasm.gz 2
add_wasm ic-icrc1-ledger.wasm.gz 3
add_wasm sns-swap-canister.wasm.gz 4
add_wasm ic-icrc1-archive.wasm.gz 5
add_wasm ic-icrc1-index.wasm.gz 6

