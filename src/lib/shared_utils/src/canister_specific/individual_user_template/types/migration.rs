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
