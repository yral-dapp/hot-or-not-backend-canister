# !/bin/bash


dfx build platform_orchestrator --network=ic

dfx canister stop platform_orchestrator --network=ic

dfx canister install platform_orchestrator  --argument "(record {version= \"v2.2.0\"})" --mode=upgrade --network=ic 


dfx canister start platform_orchestrator --network=ic