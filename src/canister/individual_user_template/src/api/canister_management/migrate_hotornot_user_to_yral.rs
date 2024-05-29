use crate::util::migration::{IndividualUser, Migration};
use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::{
    migration::MigrationErrors, post::Post,
};
use std::collections::BTreeMap;

#[update]
pub async fn transfer_tokens_and_posts(
    to_account: Principal,
    to_account_canister_id: Principal,
) -> Result<(), MigrationErrors> {
    let caller = caller();
    let user = IndividualUser::from_canister_data().await?;
    let to_individual_user = IndividualUser::new(to_account_canister_id, to_account, None).await?;
    user.transfer_tokens_and_posts(caller, to_individual_user)
        .await
}

#[update]
pub async fn receive_data_from_hotornot(
    from_account: Principal,
    amount: u64,
    posts: BTreeMap<u64, Post>,
) -> Result<(), MigrationErrors> {
    let user = IndividualUser::from_canister_data().await?;

    let from_individual_user = IndividualUser::new(caller(), from_account, None).await?;

    user.recieve_tokens_and_posts(from_individual_user, amount, posts)?;
    Ok(())
}
