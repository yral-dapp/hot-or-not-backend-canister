use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use shared_utils::{
    access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMap,
};

#[derive(Default, CandidType, Deserialize)]
pub struct CanisterData {
    pub known_principal_ids: KnownPrincipalMap,
    pub access_control_list: HashMap<Principal, Vec<UserAccessRole>>,
    pub signups_enabled: bool,
}
