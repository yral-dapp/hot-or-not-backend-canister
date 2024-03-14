# !/bin/bash


dfx build individual_user_template

# Specify the path to your Wasm.gz file
wasm=".dfx/local/canisters/individual_user_template/individual_user_template.wasm.gz"



# Display the hexdump or use the variable as needed
# echo "$(hexdump -ve '1/1 "%.2x"' "$wasm" | sed 's/../\\&/g')"


# dfx canister install platform_orchestrator --mode=reinstall --argument "(record {
#   user_index_wasm = null;
#   subnet_orchestrator_wasm = null;
#   version= \"v1.0.0\"
# })"

# Use xxd to convert the file content to a hexadecimal string
char=$(hexdump -ve '1/1 "%.2x"' "$wasm")

# Escape special characters in the hexadecimal string
char_escaped=$(printf "%s" "$char" | sed 's/../\\&/g')

# Create a shell script with the escaped hexadecimal string
printf "(\"v2.2.0\", blob \"%s\")"  "$char_escaped" > argument
dfx canister call user_index  start_upgrades_for_individual_canisters --argument-file argument
