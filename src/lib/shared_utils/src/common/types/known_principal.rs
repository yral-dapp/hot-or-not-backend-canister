use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use speedy::{Readable, Writable};

#[derive(
    CandidType, Deserialize, PartialEq, Eq, Hash, Readable, Writable, Clone, Serialize, Debug,
)]
pub enum KnownPrincipalType {
    UserIdGlobalSuperAdmin,
    CanisterIdConfiguration,
    CanisterIdDataBackup,
    CanisterIdPostCache,
    CanisterIdProjectMemberIndex,
    CanisterIdRootCanister,
    CanisterIdSNSController,
    CanisterIdTopicCacheIndex,
    CanisterIdUserIndex,
}

// TODO: Migrate implementers to V1
// pub type KnownPrincipalMap = HashMap<KnownPrincipalType, SPrincipal>;
pub type KnownPrincipalMap = HashMap<KnownPrincipalType, Principal>;
