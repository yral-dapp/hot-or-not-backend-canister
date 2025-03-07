use candid::Principal;
use serde::{Deserialize, Serialize};

use crate::CANISTER_DATA;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq)]
pub(crate) enum SubnetOrchestratorOperation {
    RechargeIndividualUserCanister(Principal),
}
