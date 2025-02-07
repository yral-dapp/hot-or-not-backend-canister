use std::collections::HashMap;

use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct DeployedCdaoCanisters {
    pub governance: Principal,
    pub ledger: Principal,
    pub root: Principal,
    pub swap: Principal,
    pub index: Principal,

    #[serde(default)]
    pub airdrop_info: AirdropInfo,

    #[serde(default)]
    pub last_swapped_price: Option<f64>
}

impl DeployedCdaoCanisters {
    pub fn get_canister_ids(&self) -> Vec<Principal> {
        vec![
            self.governance,
            self.ledger,
            self.root,
            self.swap,
            self.index,
        ]
    }
}

#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct AirdropInfo {
    /// Maps each principal to their claim status
    #[serde(default)]
    pub principals_who_successfully_claimed: HashMap<Principal, ClaimStatus>,
}

impl AirdropInfo {
    pub fn get_claim_status(&self, user_principal_id: &Principal) -> Result<ClaimStatus, String> {
        self.principals_who_successfully_claimed
            .get(user_principal_id)
            .cloned()
            .ok_or_else(|| format!("Principal {} not found", user_principal_id))
    }

    pub fn is_airdrop_claimed(&self, user_principal_id: &Principal) -> Result<bool, String> {
        match self.get_claim_status(user_principal_id)? {
            ClaimStatus::Claimed => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn is_airdrop_claiming(&self, user_principal_id: &Principal) -> Result<bool, String> {
        match self.get_claim_status(user_principal_id)? {
            ClaimStatus::Claiming => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn is_airdrop_unclaimed(&self, user_principal_id: &Principal) -> bool {
        matches!(
            self.get_claim_status(user_principal_id),
            Ok(ClaimStatus::Unclaimed) | Err(_)
        )
    }

    fn set_claim_status_or_insert_with_claim_status_if_not_exist(
        &mut self,
        user_principal_id: &Principal,
        status: ClaimStatus,
    ) {
        use std::collections::hash_map::Entry;

        match self
            .principals_who_successfully_claimed
            .entry(*user_principal_id)
        {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = status;
            }
            Entry::Vacant(entry) => {
                entry.insert(status);
            }
        }
    }

    pub fn set_airdrop_claimed(&mut self, user_principal_id: Principal) {
        self.set_claim_status_or_insert_with_claim_status_if_not_exist(
            &user_principal_id,
            ClaimStatus::Claimed,
        )
    }

    pub fn set_airdrop_claiming(&mut self, user_principal_id: Principal) {
        self.set_claim_status_or_insert_with_claim_status_if_not_exist(
            &user_principal_id,
            ClaimStatus::Claiming,
        )
    }

    pub fn set_airdrop_unclaimed(&mut self, user_principal_id: Principal) {
        self.set_claim_status_or_insert_with_claim_status_if_not_exist(
            &user_principal_id,
            ClaimStatus::Unclaimed,
        )
    }
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub enum ClaimStatus {
    #[default]
    Unclaimed,
    Claimed,
    Claiming,
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub struct SwapTokenData{
    pub ledger: Principal,
    pub amt: Nat
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub struct TokenPairs{
    pub token_a: SwapTokenData,
    pub token_b: SwapTokenData
}
