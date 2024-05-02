use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Copy, CandidType, Debug, PartialEq, Eq)]
pub enum MigrationInfo {
    #[default]
    NotMigrated,
    MigratedFromHotOrNot {
        to_yral_principal_id: Principal,
    },
    MigratedToYral {
        from_hotornot_principal_id: Principal,
    },
}
