use std::collections::HashSet;

use serde::{Serialize, Deserialize};
use shared_utils::canister_specific::individual_user_template::types::airdrop::AirdropMember;


#[derive(Default, Deserialize, Serialize)]
pub struct AirdropData {
    #[serde(default)]
    pub parent: Option<AirdropMember>,
    #[serde(default)]
    pub token_chain: HashSet<AirdropMember>,
}
