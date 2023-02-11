use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use shared_utils::{
    access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMapV1,
};

#[derive(Default, CandidType, Deserialize, Serialize)]
pub struct HeapData {
    pub known_principal_ids: KnownPrincipalMapV1,
    pub access_control_list: HashMap<Principal, Vec<UserAccessRole>>,
}
