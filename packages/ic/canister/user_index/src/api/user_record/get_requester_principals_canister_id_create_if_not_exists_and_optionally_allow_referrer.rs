use crate::{util::canister_management::create_users_canister, CANISTER_DATA};
use candid::Principal;
use ic_cdk::api::call;

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
async fn get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer(
    referrer: Option<Principal>,
) -> Principal {
    let api_caller = ic_cdk::caller();

    if api_caller == Principal::anonymous() {
        panic!("Anonymous principal is not allowed to call this method");
    }

    let canister_id_for_this_caller = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .get(&api_caller)
            .cloned()
    });

    match canister_id_for_this_caller {
        // * canister already exists
        Some(canister_id) => canister_id,
        None => {
            // * create new canister
            let created_canister_id = create_users_canister(api_caller).await;

            CANISTER_DATA.with(|canister_data_ref_cell| {
                canister_data_ref_cell
                    .borrow_mut()
                    .user_principal_id_to_canister_id_map
                    .insert(api_caller, created_canister_id);
            });

            // * reward user for signing up
            call::notify(created_canister_id, "get_rewarded_for_signing_up", ()).ok();

            // * reward referrer for referring
            if let Some(referrer_principal_id) = referrer {
                let referrer_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
                    canister_data_ref_cell
                        .borrow()
                        .user_principal_id_to_canister_id_map
                        .get(&referrer_principal_id)
                        .cloned()
                });
                if let Some(referrer_canister_id) = referrer_canister_id {
                    call::notify(
                        referrer_canister_id,
                        "get_rewarded_for_referral",
                        (referrer_principal_id, api_caller),
                    )
                    .ok();
                    call::notify(
                        created_canister_id,
                        "get_rewarded_for_referral",
                        (referrer_principal_id, api_caller),
                    )
                    .ok();
                }
            }

            created_canister_id
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ingress_principal_id_equality_from_different_sources() {
        assert_eq!("2vxsx-fae".to_string(), Principal::anonymous().to_text());
        assert_eq!(
            Principal::from_text("2vxsx-fae").unwrap(),
            Principal::anonymous()
        );
    }
}
