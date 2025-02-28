use crate::{data_model::CanisterData, CANISTER_DATA, PUMP_N_DUMP};
use ic_cdk_macros::init;
use shared_utils::canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs;

#[init]
fn init(init_args: IndividualUserTemplateInitArgs) {
    PUMP_N_DUMP.with_borrow_mut(|pd| {
        if let Some(onboarding_reward) = init_args.pump_dump_onboarding_reward.clone() {
            pd.onboarding_reward = onboarding_reward;
        }
    });

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut data = canister_data_ref_cell.borrow_mut();
        init_impl(init_args, &mut data);
    });
}

fn init_impl(init_args: IndividualUserTemplateInitArgs, data: &mut CanisterData) {
    init_args
        .known_principal_ids
        .unwrap_or_default()
        .iter()
        .for_each(|(principal_belongs_to, principal_id)| {
            data.known_principal_ids
                .insert(*principal_belongs_to, *principal_id);
        });

    data.profile.principal_id = init_args.profile_owner;

    data.configuration.url_to_send_canister_metrics_to = init_args.url_to_send_canister_metrics_to;

    data.version_details.version_number = init_args.upgrade_version_number.unwrap_or_default();
    data.version_details.version = init_args.version;
}

#[cfg(test)]
mod test {
    use shared_utils::common::{types::known_principal::{KnownPrincipalMap, KnownPrincipalType}, utils::default_pump_dump_onboarding_reward};
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id, get_mock_canister_id_user_index,
        get_mock_user_alice_principal_id,
    };

    use super::*;

    #[test]
    fn test_init_impl() {
        // * Add some known principals
        let mut known_principal_ids = KnownPrincipalMap::new();
        known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id(),
        );
        known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );

        // * Create the init args
        let init_args = IndividualUserTemplateInitArgs {
            known_principal_ids: Some(known_principal_ids),
            profile_owner: Some(get_mock_user_alice_principal_id()),
            upgrade_version_number: Some(0),
            url_to_send_canister_metrics_to: Some(
                "http://metrics-url.com/receive-metrics".to_string(),
            ),
            version: String::from("v1.0.0"),
            pump_dump_onboarding_reward: Some(default_pump_dump_onboarding_reward()),
        };
        let mut data = CanisterData::default();

        // * Run the init impl
        init_impl(init_args, &mut data);

        // * Check the data
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .unwrap(),
            &get_global_super_admin_principal_id()
        );
        assert_eq!(
            data.known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .unwrap(),
            &get_mock_canister_id_user_index()
        );

        assert_eq!(
            data.profile.principal_id,
            Some(get_mock_user_alice_principal_id())
        );

        assert_eq!(
            data.configuration.url_to_send_canister_metrics_to,
            Some("http://metrics-url.com/receive-metrics".to_string())
        );

        assert!(data.version_details.version.eq("v1.0.0"));
    }
}
