use std::collections::HashMap;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct DeployedCdaoCanisters {
    pub governance: Principal,
    pub ledger: Principal,
    pub root: Principal,
    pub swap: Principal,
    pub index: Principal,
    pub airdrop_info: AirdropInfo,
}
#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AirdropInfo {
    /// Maps each principal to their claim status
    pub principals_who_successfully_claimed: HashMap<Principal, ClaimStatus>,
}

impl AirdropInfo {
    pub fn get_claim_status(&self, user_id: &Principal) -> Result<ClaimStatus, String> {
        self.principals_who_successfully_claimed
            .get(user_id)
            .cloned()
            .ok_or_else(|| format!("Principal {} not found", user_id))
    }

    pub fn is_airdrop_claimed(&self, user_id: &Principal) -> Result<bool, String> {
        match self.get_claim_status(user_id)? {
            ClaimStatus::Claimed => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn is_airdrop_claiming(&self, user_id: &Principal) -> Result<bool, String> {
        match self.get_claim_status(user_id)? {
            ClaimStatus::Claiming => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn is_airdrop_unclaimed(&self, user_id: &Principal) -> bool{
        matches!(self.get_claim_status(user_id), Ok(ClaimStatus::Unclaimed))
    }

    fn set_claim_status_or_insert_with_claim_status_if_not_exist(
        &mut self,
        user_id: &Principal,
        status: ClaimStatus,
    ) {
        use std::collections::hash_map::Entry;

        match self.principals_who_successfully_claimed.entry(*user_id) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = status;
            }
            Entry::Vacant(entry) => {
                entry.insert(status);
            }
        }
    }

    pub fn set_airdrop_claimed(&mut self, user_id: Principal) {
        self.set_claim_status_or_insert_with_claim_status_if_not_exist(&user_id, ClaimStatus::Claimed)
    }

    pub fn set_airdrop_claiming(&mut self, user_id: Principal){
        self.set_claim_status_or_insert_with_claim_status_if_not_exist(&user_id, ClaimStatus::Claiming)
    }

    pub fn set_airdrop_unclaimed(&mut self, user_id: Principal) {
        self.set_claim_status_or_insert_with_claim_status_if_not_exist(&user_id, ClaimStatus::Unclaimed)
    }
}

#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct PrincipalEligibleToClaimAirdrop {
    pub user_id: Principal,
    pub claim_status: ClaimStatus,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug, PartialEq, Eq, Default, Hash)]
pub enum ClaimStatus {
    #[default]
    Unclaimed,
    Claimed,
    Claiming,
}