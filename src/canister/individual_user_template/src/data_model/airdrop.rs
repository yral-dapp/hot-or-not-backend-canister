use candid::Principal;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
pub struct AirdropData {
    pub parent: Option<Principal>,
}

impl Default for AirdropData {
    fn default() -> Self {
        Self {
            parent: None,
        }
    }
}