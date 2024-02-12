# !/bin/bash


cargo test --package platform_orchestrator
dfx build platform_orchestrator --network=ic

dfx canister install platform_orchestrator  --argument "(record {version= \"v2.0.0\"})" --mode=upgrade --network=ic --identity ravibazz-ic