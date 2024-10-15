use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk::update;
use ic_sns_root::pb::v1::{ListSnsCanistersRequest, ListSnsCanistersResponse};
use icrc_ledger_types::icrc1::{account::Account, transfer::{Memo, TransferArg, TransferError}};
use shared_utils::canister_specific::individual_user_template::types::{error::CdaoTokenError, profile::UserProfileDetailsForFrontendV2};

use crate::CANISTER_DATA;

#[update]
async fn request_airdrop(token_root: Principal, memo: Option<Memo>, amount: Nat, user_canister: Principal) -> Result<(), CdaoTokenError> {
    let current_caller = ic_cdk::caller();
    let profile_info = get_profile_info(user_canister).await?;
    
    if profile_info.principal_id != current_caller {
        return Err(CdaoTokenError::Unauthenticated);
    }

    if !is_airdrop_unclaimed(token_root, &current_caller)? {// assertion is checked here
        return Ok(());
    }

    let amount = amount.min(1000u32.into());
    if amount < 100u32 {
        return Ok(());
    }

    set_airdrop_claiming(token_root, current_caller); // can safely ignore error here assertion is already checked

    request_airdrop_internal(token_root, current_caller, memo, amount).await.inspect(|_|{
        CANISTER_DATA.with_borrow_mut(|cans_data| {
            cans_data
                .cdao_canisters
                .iter_mut()
                .find(|cdao| cdao.root == token_root)
                .map(|cdao| cdao.airdrop_info.set_airdrop_unclaimed(current_caller)).unwrap(); // can safely unwrap updating the states for the airdrop for the user creates it in place if not exists
        });
    })?; // rollback to unclaimed if error

    set_airdrop_claimed(token_root, current_caller); // can safely ignore error here assertion is already checked

    Ok(())
}


async fn request_airdrop_internal(token_root: Principal, current_caller: Principal, memo: Option<Memo>, amount: Nat) -> Result<(), CdaoTokenError> {
    let ledger = get_ledger(token_root).await?;
    let balance = get_balance(ledger.into()).await?;
    
    if balance < amount {
        return Ok(());
    }

    transfer_tokens(ledger.into(), current_caller, memo, amount).await?;
    Ok(())
}
async fn get_profile_info(user_canister: Principal) -> Result<UserProfileDetailsForFrontendV2, CdaoTokenError> {
    let (profile_info,): (UserProfileDetailsForFrontendV2,) = ic_cdk::call(user_canister, "get_profile_details_v2", ()).await?;
    Ok(profile_info)
}

fn is_airdrop_unclaimed(token_root: Principal, current_caller: &Principal) -> Result<bool, CdaoTokenError> {
    CANISTER_DATA.with_borrow(|cans_data| {
        cans_data.cdao_canisters.iter().find(|cdao| cdao.root == token_root)
            .map(|cdao| cdao.airdrop_info.is_airdrop_unclaimed(current_caller))
    }).ok_or(CdaoTokenError::InvalidRoot)?.map_err(|_| CdaoTokenError::Unauthenticated)
}

fn set_airdrop_claiming(token_root: Principal, current_caller: Principal) {
    CANISTER_DATA.with_borrow_mut(|cans_data| {
        if let Some(cdao) = cans_data.cdao_canisters.iter_mut().find(|cdao| cdao.root == token_root) {
            cdao.airdrop_info.set_airdrop_claiming(current_caller)
        }
    })
}

async fn get_ledger(token_root: Principal) -> Result<PrincipalId, CdaoTokenError> {
    let res: (ListSnsCanistersResponse,) = ic_cdk::call(token_root, "list_sns_canisters", (ListSnsCanistersRequest {},)).await?;
    res.0.ledger.ok_or(CdaoTokenError::InvalidRoot)
}

async fn get_balance(ledger: Principal) -> Result<Nat, CdaoTokenError> {
    let account = Account { owner: ic_cdk::id(), subaccount: None };
    let (balance_res,): (Nat,) = ic_cdk::call(ledger, "icrc1_balance_of", (account,)).await?;
    Ok(balance_res)
}

async fn transfer_tokens(ledger: Principal, current_caller: Principal, memo: Option<Memo>, amount: Nat) -> Result<(), CdaoTokenError> {
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: current_caller, subaccount: None },
        fee: None,
        created_at_time: None,
        memo,
        amount,
    };
    let transfer_res: (Result<Nat, TransferError>,) = ic_cdk::call(ledger, "icrc1_transfer", (transfer_args,)).await?;
    let _ = transfer_res.0.map_err(CdaoTokenError::Transfer)?;
    Ok(())
}

fn set_airdrop_claimed(token_root: Principal, current_caller: Principal) {
    CANISTER_DATA.with_borrow_mut(|cans_data| {
        if let Some(cdao) = cans_data.cdao_canisters.iter_mut().find(|cdao| cdao.root == token_root) {
            cdao.airdrop_info.set_airdrop_claimed(current_caller)
        }
    })
}

// Assuming the original code is in a module named `airdrop`
#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use shared_utils::canister_specific::individual_user_template::types::cdao::{AirdropInfo, ClaimStatus};
    use std::collections::HashMap;

    /// Helper function to create a mock Principal from a string.
    fn mock_principal(id: &str) -> Principal {
        Principal::from_slice(id.as_bytes())
    }

    #[test]
    fn test_get_claim_status_existing_principal() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user1");
        principals_map.insert(principal.clone(), ClaimStatus::Claimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let status = airdrop_info.get_claim_status(&principal).unwrap();

        // Assert
        assert_eq!(status, ClaimStatus::Claimed);
    }

    #[test]
    fn test_get_claim_status_non_existing_principal() {
        // Arrange
        let principals_map = HashMap::new();
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };
        let principal = mock_principal("user2");

        // Act
        let result = airdrop_info.get_claim_status(&principal);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Principal {} not found", principal)
        );
    }

    #[test]
    fn test_is_airdrop_claimed_true() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user3");
        principals_map.insert(principal.clone(), ClaimStatus::Claimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let result = airdrop_info.is_airdrop_claimed(&principal).unwrap();

        // Assert
        assert!(result);
    }

    #[test]
    fn test_is_airdrop_claimed_false() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user4");
        principals_map.insert(principal.clone(), ClaimStatus::Claiming);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let result = airdrop_info.is_airdrop_claimed(&principal).unwrap();

        // Assert
        assert!(!result);
    }

    #[test]
    fn test_is_airdrop_claiming_true() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user5");
        principals_map.insert(principal.clone(), ClaimStatus::Claiming);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let result = airdrop_info.is_airdrop_claiming(&principal).unwrap();

        // Assert
        assert!(result);
    }

    #[test]
    fn test_is_airdrop_claiming_false() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user6");
        principals_map.insert(principal.clone(), ClaimStatus::Unclaimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let result = airdrop_info.is_airdrop_claiming(&principal).unwrap();

        // Assert
        assert!(!result);
    }

    #[test]
    fn test_is_airdrop_unclaimed_true() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user7");
        principals_map.insert(principal.clone(), ClaimStatus::Unclaimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let result = airdrop_info.is_airdrop_unclaimed(&principal).unwrap();

        // Assert
        assert!(result);
    }

    #[test]
    fn test_is_airdrop_unclaimed_false() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user8");
        principals_map.insert(principal.clone(), ClaimStatus::Claimed);
        let airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        let result = airdrop_info.is_airdrop_unclaimed(&principal).unwrap();

        // Assert
        assert!(!result);
    }

    #[test]
    fn test_set_airdrop_claimed_new_principal() {
        // Arrange
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal = mock_principal("user9");

        // Act
        airdrop_info.set_airdrop_claimed(principal.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claimed)
        );
    }

    #[test]
    fn test_set_airdrop_claimed_existing_principal() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user10");
        principals_map.insert(principal.clone(), ClaimStatus::Unclaimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        airdrop_info.set_airdrop_claimed(principal.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claimed)
        );
    }

    #[test]
    fn test_set_airdrop_claiming_new_principal() {
        // Arrange
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal = mock_principal("user11");

        // Act
        airdrop_info.set_airdrop_claiming(principal.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claiming)
        );
    }

    #[test]
    fn test_set_airdrop_claiming_existing_principal() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user12");
        principals_map.insert(principal.clone(), ClaimStatus::Unclaimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        airdrop_info.set_airdrop_claiming(principal.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Claiming)
        );
    }

    #[test]
    fn test_set_airdrop_unclaimed_new_principal() {
        // Arrange
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        };
        let principal = mock_principal("user13");

        // Act
        airdrop_info.set_airdrop_unclaimed(principal.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Unclaimed)
        );
    }

    #[test]
    fn test_set_airdrop_unclaimed_existing_principal() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user14");
        principals_map.insert(principal.clone(), ClaimStatus::Claimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        airdrop_info.set_airdrop_unclaimed(principal.clone());

        // Assert
        assert_eq!(
            airdrop_info.principals_who_successfully_claimed.get(&principal),
            Some(&ClaimStatus::Unclaimed)
        );
    }

    #[test]
    fn test_update_existing_principal_status() {
        // Arrange
        let mut principals_map = HashMap::new();
        let principal = mock_principal("user15");
        principals_map.insert(principal.clone(), ClaimStatus::Unclaimed);
        let mut airdrop_info = AirdropInfo {
            principals_who_successfully_claimed: principals_map,
        };

        // Act
        airdrop_info.set_airdrop_claiming(principal.clone());
        airdrop_info.set_airdrop_claimed(principal.clone());

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