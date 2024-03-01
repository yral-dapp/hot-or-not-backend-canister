use std::borrow::Cow;

use candid::{candid_method, types::principal, CandidType};
use ic_cdk::api;
use ic_stable_structures::Storable;
use serde::Deserialize;

use crate::CANISTER_DATA;

#[derive(CandidType, Deserialize)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

fn get_path(url: &str) -> Option<&str> {
    url.split('?').next()
}

#[cfg(target_arch = "wasm32")]
const WASM_PAGE_SIZE: u64 = 65536;

pub fn get_stable_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (ic_cdk::api::stable::stable64_size() as u64) * WASM_PAGE_SIZE
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn get_heap_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (core::arch::wasm32::memory_size(0) as u64) * WASM_PAGE_SIZE
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

fn metrics() -> String {
    // Prometheus expects timestamps in ms. Time.now() returns ns.
    let timestamp = api::time() / 1000000;
    let version =
        CANISTER_DATA.with(|canister_data| canister_data.borrow().version_details.version_number);
    let principal =
        match CANISTER_DATA.with(|canister_data| canister_data.borrow().profile.principal_id) {
            Some(principal) => principal.to_string(),
            None => "None".to_string(),
        };

    vec![
        format!("cycle_balance {} {}", api::canister_balance128(), timestamp).as_str(),
        format!("heap_size {} {}", get_heap_memory_size(), timestamp).as_str(),
        format!("stable_size {} {}", get_stable_memory_size(), timestamp).as_str(),
        format!("version {} {}", version, timestamp).as_str(),
        format!("profile{{principal=\"{}\"}} 1 {}", principal, timestamp).as_str(),
    ]
    .connect("\n")
}

fn retrieve(path: &str) -> Option<Vec<u8>> {
    match path {
        "/metrics" => Some(metrics().as_bytes().to_vec()),
        _ => None,
    }
}

#[ic_cdk_macros::query]
fn http_request(request: HttpRequest) -> HttpResponse {
    let path = get_path(request.url.as_str()).unwrap_or("/");
    if let Some(bytes) = retrieve(path) {
        HttpResponse {
            status_code: 200,
            headers: vec![
                //HeaderField("Content-Encoding".to_string(), "gzip".to_string()),
                HeaderField("Content-Length".to_string(), format!("{}", bytes.len())),
                HeaderField("Cache-Control".to_string(), format!("max-age={}", 600)),
                HeaderField("Content-Type".to_string(), "text/plain".to_string()),
            ],
            body: bytes,
        }
    } else {
        HttpResponse {
            status_code: 404,
            headers: Vec::new(),
            body: path.as_bytes().to_vec(),
        }
    }
}
