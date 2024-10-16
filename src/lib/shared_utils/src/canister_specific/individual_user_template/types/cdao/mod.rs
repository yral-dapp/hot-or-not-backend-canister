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

    pub fn is_airdrop_unclaimed(&self, user_id: &Principal) -> Result<bool, String> {
        match self.get_claim_status(user_id)? {
            ClaimStatus::Unclaimed => Ok(true),
            _ => Ok(false),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use std::collections::HashMap;

    fn mock_principal(id: &str) -> Principal {
        Principal::from_slice(id.as_bytes())
    }

    #[test]
    fn test_get_claim_status_existing_principal() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user1");
        principals_map.insert(principal, ClaimStatus::Claimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let status = airdrop_info.get_claim_status(&principal).unwrap();

        assert_eq!(status, ClaimStatus::Claimed);
    }

    #[test]
    fn test_get_claim_status_non_existing_principal() {
        let principals_map = HashMap::new();
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };
        let principal = mock_principal("user2");

        let result = airdrop_info.get_claim_status(&principal);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Principal {} not found", principal)
        );
    }

    #[test]
    fn test_is_airdrop_claimed_true() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user3");
        principals_map.insert(principal, ClaimStatus::Claimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let result = airdrop_info.is_airdrop_claimed(&principal).unwrap();

        assert!(result);
    }

    #[test]
    fn test_is_airdrop_claimed_false() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user4");
        principals_map.insert(principal, ClaimStatus::Claiming);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let result = airdrop_info.is_airdrop_claimed(&principal).unwrap();

        assert!(!result);
    }

    #[test]
    fn test_is_airdrop_claiming_true() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user5");
        principals_map.insert(principal, ClaimStatus::Claiming);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let result = airdrop_info.is_airdrop_claiming(&principal).unwrap();

        assert!(result);
    }

    #[test]
    fn test_is_airdrop_claiming_false() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user6");
        principals_map.insert(principal, ClaimStatus::Unclaimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let result = airdrop_info.is_airdrop_claiming(&principal).unwrap();

        assert!(!result);
    }

    #[test]
    fn test_is_airdrop_unclaimed_true() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user7");
        principals_map.insert(principal, ClaimStatus::Unclaimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let result = airdrop_info.is_airdrop_unclaimed(&principal).unwrap();

        assert!(result);
    }

    #[test]
    fn test_is_airdrop_unclaimed_false() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user8");
        principals_map.insert(principal, ClaimStatus::Claimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        let result = airdrop_info.is_airdrop_unclaimed(&principal).unwrap();

        assert!(!result);
    }

    #[test]
    fn test_set_airdrop_claimed_new_principal() {
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal = mock_principal("user9");

        airdrop_info.set_airdrop_claimed(principal);

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claimed)
        );
    }

    #[test]
    fn test_set_airdrop_claimed_existing_principal() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user10");
        principals_map.insert(principal, ClaimStatus::Unclaimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        airdrop_info.set_airdrop_claimed(principal);

        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claimed)
        );
    }

    #[test]
    fn test_set_airdrop_claiming_new_principal() {
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal = mock_principal("user11");

        airdrop_info.set_airdrop_claiming(principal);

        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claiming)
        );
    }

    #[test]
    fn test_set_airdrop_claiming_existing_principal() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user12");
        principals_map.insert(principal, ClaimStatus::Unclaimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        airdrop_info.set_airdrop_claiming(principal);

        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claiming)
        );
    }

    #[test]
    fn test_set_airdrop_unclaimed_new_principal() {
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal = mock_principal("user13");

        airdrop_info.set_airdrop_unclaimed(principal);

        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Unclaimed)
        );
    }

    #[test]
    fn test_set_airdrop_unclaimed_existing_principal() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user14");
        principals_map.insert(principal, ClaimStatus::Claimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        airdrop_info.set_airdrop_unclaimed(principal);

        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Unclaimed)
        );
    }

    #[test]
    fn test_update_existing_principal_status() {
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user15");
        principals_map.insert(principal, ClaimStatus::Unclaimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        airdrop_info.set_airdrop_claiming(principal);
        airdrop_info.set_airdrop_claimed(principal);

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claimed)
        );
    }

    #[test]
    fn test_multiple_principals() {
        // Arrange
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal1 = mock_principal("user16");
        let principal2 = mock_principal("user17");

        // Act
        airdrop_info.set_airdrop_claimed(principal1.clone());
        airdrop_info.set_airdrop_claiming(principal2.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal1),
            Some(&ClaimStatus::Claimed)
        );
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal2),
            Some(&ClaimStatus::Claiming)
        );
    }
}