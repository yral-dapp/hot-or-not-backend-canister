# !/bin/bash


dfx build user_index

# Specify the path to your Wasm.gz file
wasm=".dfx/local/canisters/user_index/user_index.wasm.gz"



# Use xxd to convert the file content to a hexadecimal string
char=$(hexdump -ve '1/1 "%.2x"' "$wasm")

# Escape special characters in the hexadecimal string
char_escaped=$(printf "%s" "$char" | sed 's/../\\&/g')

# Create a shell script with the escaped hexadecimal string
printf "(record {version = \"v2.2.0\"; canister = variant {SubnetOrchestratorWasm}; wasm_blob = blob \"%s\"})"  "$char_escaped" > argument
dfx canister call platform_orchestrator  upgrade_canisters_in_network --argument-file argument