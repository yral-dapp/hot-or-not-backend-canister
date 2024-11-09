DFX_IC_COMMIT=e54d3fa34ded227c885d04e64505fa4b5d564743
CANISTER_NAME=ic-icrc1-index-ng.wasm.gz

if [ ! -e "./wams/$CANISTER_NAME" ]; then
    wget -P ./wasms/ "https://download.dfinity.systems/ic/$DFX_IC_COMMIT/canisters/$CANISTER_NAME"
fi

