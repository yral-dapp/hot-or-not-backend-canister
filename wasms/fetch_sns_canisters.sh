DFX_IC_COMMIT=aa91ecacdf3824e193e21b70e0127e8d3edab51a
CANISTER_NAME=sns-root-canister.wasm.gz

if [ ! -e "./wams/$CANISTER_NAME" ]; then
    wget -P ./wasms/ "https://download.dfinity.systems/ic/$DFX_IC_COMMIT/canisters/$CANISTER_NAME"
fi

# archive canister commit: e54d3fa34ded227c885d04e64505fa4b5d564743
# ledger canister commit: e54d3fa34ded227c885d04e64505fa4b5d564743
# swap canister commit: aa91ecacdf3824e193e21b70e0127e8d3edab51a
# governance canister commit: 25c1bb0227d9970f5673b908817d7c4962b29911
# root canister commit: aa91ecacdf3824e193e21b70e0127e8d3edab51a
# index canister commit: e54d3fa34ded227c885d04e64505fa4b5d564743



