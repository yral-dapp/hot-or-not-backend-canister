use candid::CandidType;
use serde::Deserialize;



#[derive(CandidType, Deserialize)]
pub struct PlatformOrchestratorInitArgs {
    pub version: String,
}