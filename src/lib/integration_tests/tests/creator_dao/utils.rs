use candid::{Encode, Principal};
use ic_sns_wasm::init::SnsWasmCanisterInitPayload;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::constant::SNS_WASM_W_PRINCIPAL_ID;

use crate::{add_wasm, AddWasmResultRecord};

pub fn setup_sns_w_canister_for_creator_dao(pocket_ic: &PocketIc, super_admin: Principal) {
    let sns_wasm_w_canister_wasm = include_bytes!("../../../../../wasms/sns-wasm-canister.wasm");
    let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

    let _ = pocket_ic.create_canister_with_id(
        Some(super_admin),
        None,
        Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap(),
    );

    let sns_wasm_canister_init_payload = SnsWasmCanisterInitPayload {
        sns_subnet_ids: vec![],
        access_controls_enabled: false,
        allowed_principals: vec![],
    };

    pocket_ic.install_canister(
        sns_wasm_w_canister_id,
        sns_wasm_w_canister_wasm.to_vec(),
        Encode!(&sns_wasm_canister_init_payload).unwrap(),
        Some(super_admin),
    );

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/root.wasm.gz"),
                1,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/governance.wasm.gz"),
                2,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/ledger.wasm.gz"),
                3,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/swap.wasm.gz"),
                4,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/archive.wasm.gz"),
                5,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/index.wasm.gz"),
                6,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    for _ in 0..50 {
        pocket_ic.tick();
    }
}
