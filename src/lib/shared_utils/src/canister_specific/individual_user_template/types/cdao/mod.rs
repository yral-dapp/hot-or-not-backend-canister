use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct DeployedCdaoCanisters {
    pub governance: Principal,
    pub ledger: Principal,
    pub root: Principal,
    pub swap: Principal,
    pub index: Principal,
}
