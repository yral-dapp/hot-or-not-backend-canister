use std::borrow::Cow;

use candid::{candid_method, CandidType};
use ic_cdk::api;
use ic_stable_structures::Storable;
use serde::Deserialize;

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

fn retrieve(path: &str) -> Option<Vec<u8>> {
    match path {
        "/metrics" => Some(api::canister_balance128().to_le_bytes().to_vec()),
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
