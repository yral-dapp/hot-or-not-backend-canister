use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::common::types::known_principal::KnownPrincipalMap;

#[derive(Deserialize, CandidType, Default)]
pub struct PostCacheInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
    pub upgrade_version_number: Option<u64>,
    pub version: String,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub enum NsfwFilter {
    ExcludeNsfw,
    OnlyNsfw,
    IncludeNsfw,
}
