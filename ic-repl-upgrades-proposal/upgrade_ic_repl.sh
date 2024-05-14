#!/usr/bin/ic-repl -o
import platform_orchestrator_canister = "74zq4-iqaaa-aaaam-ab53a-cai";
import governance_canister="6wcax-haaaa-aaaaq-aaava-cai";

function generate_payload() {
    let canister_type = variant {IndividualUserWasm};

    if eq("${CANISTER_NAME}", "post_cache") {
        let canister_type = variant {PostCacheWasm};
    } else {
    };
    if eq("${CANISTER_NAME}", "user_index") {
        let canister_type = variant {SubnetOrchestratorWasm};
    } else {
        
    };
    if eq("${CANISTER_NAME}", "individual_user_template") {
        let canister_type = variant {IndividualUserWasm};
    } else {
        
    };

    encode platform_orchestrator_canister.platform_orchestrator_generic_function(
            variant {
                UpgradeSubnetCanisters = record {
                    version = "v1.0.0"; 
                    canister = canister_type; 
                    wasm_blob = file("../.dfx/ic/canisters/${CANISTER_NAME}/${CANISTER_NAME}.wasm.gz");
                }
            }
    )
   
};


let summary = "${SUMMARY}";
let res = encode governance_canister.manage_neuron(
    record {
        subaccount = ("a990d55e65186d5b4f38912c9a10748bf2e6626fb582a2b1d6442e087db56b10": blob);
        command = opt variant {MakeProposal = record {
            title = "upgrade canister";
            url = "yral.com";
            summary = summary;
            action = opt variant {ExecuteGenericNervousSystemFunction = record {
                function_id = 4002;
                payload = generate_payload();
            }}
        }}
    }
)




