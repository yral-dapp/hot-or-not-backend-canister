use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Copy, CandidType, Debug, PartialEq, Eq)]
pub enum MigrationInfo {
    #[default]
    NotMigrated,
    MigratedFromHotOrNot {
        account_principal: Principal,
    },
    MigratedToYral {
        account_principal: Principal,
    },
}

#[derive(Serialize, Deserialize, Debug, CandidType, PartialEq)]
pub enum MigrationErrors {
    InvalidToCanister,
    InvalidFromCanister,
    MigrationInfoNotFound,
    AlreadyMigrated,
    AlreadyUsedForMigration,
    TransferToCanisterCallFailed(String),
    CanisterInfoFailed,
    UserNotRegistered,
    Unauthorized,
    HotOrNotSubnetCanisterIdNotFound,
}
