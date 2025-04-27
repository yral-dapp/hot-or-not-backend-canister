mod mock_ledger;

use candid::{Nat, Principal};
use mock_ledger::{
    mock_ledger_intf::{Account, ApproveArgs, TransferArg},
    LEDGER_FEE, LEDGER_MINT_AMOUNT,
};
use pocket_ic::PocketIc;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        pump_n_dump::{
            BalanceInfo, GameDirection, ParticipatedGameInfo, PumpNDumpStateDiff, PumpsAndDumps,
        },
        session::SessionType,
    },
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
    constant::{GDOLLR_TO_E8S, GLOBAL_SUPER_ADMIN_USER_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::{
        execute_query, execute_update, execute_update_no_res, execute_update_no_res_multi,
        get_new_pocket_ic_env,
    },
    test_constants::{
        get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    },
};

struct PumpNDumpHarness {
    pic: PocketIc,
    known_principals: KnownPrincipalMap,
    user_index: Principal,
}

impl Default for PumpNDumpHarness {
    fn default() -> Self {
        let (pic, mut known_principals) = get_new_pocket_ic_env();

        let platform_canister_id =
            known_principals[&KnownPrincipalType::CanisterIdPlatformOrchestrator];

        let super_admin = known_principals
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .copied()
            .unwrap();
        let charlie_global_admin = get_mock_user_charlie_principal_id();

        execute_update_no_res(
            &pic,
            super_admin,
            platform_canister_id,
            "add_principal_as_global_admin",
            &charlie_global_admin,
        );

        let ckbtc_ledger = mock_ledger::deploy(&pic, charlie_global_admin);
        execute_update_no_res_multi(
            &pic,
            super_admin,
            platform_canister_id,
            "update_global_known_principal",
            (KnownPrincipalType::CanisterIdCkBTCLedger, ckbtc_ledger),
        );
        known_principals.insert(KnownPrincipalType::CanisterIdCkBTCLedger, ckbtc_ledger);

        let ckbtc_treasury = get_mock_user_dan_principal_id();
        execute_update_no_res(
            &pic,
            charlie_global_admin,
            ckbtc_ledger,
            "icrc1_transfer",
            &TransferArg {
                to: Account {
                    owner: ckbtc_treasury,
                    subaccount: None,
                },
                fee: None,
                memo: None,
                from_subaccount: None,
                created_at_time: None,
                amount: (LEDGER_MINT_AMOUNT - LEDGER_FEE).into(),
            },
        );
        execute_update_no_res_multi(
            &pic,
            super_admin,
            platform_canister_id,
            "update_global_known_principal",
            (KnownPrincipalType::UserIdCkBTCTreasury, ckbtc_treasury),
        );
        
        known_principals.insert(KnownPrincipalType::UserIdCkBTCTreasury, ckbtc_treasury);

        let app_subnets = pic.topology().get_app_subnets();

        let subnet_orchestartor: Principal = execute_update::<_, Result<_, String>>(
            &pic,
            charlie_global_admin,
            platform_canister_id,
            "provision_subnet_orchestrator_canister",
            &app_subnets[1],
        )
        .unwrap();

        for _ in 0..50 {
            pic.tick();
        }

        Self {
            pic,
            known_principals,
            user_index: subnet_orchestartor,
        }
    }
}

impl PumpNDumpHarness {
    pub fn provision_individual_canister(&self, owner: Principal) -> Principal {
        let new_canister: Principal = execute_update::<_, Result<_, String>>(
            &self.pic,
            owner,
            self.user_index,
            "get_requester_principals_canister_id_create_if_not_exists",
            &(),
        )
        .unwrap();

        // XX: hack, this is a canister bug, where individual user canister uses mainnet admin id, even in testing
        // for certain update calls
        let super_admin = Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap();
        let update_res = execute_update::<_, Result<String, String>>(
            &self.pic,
            super_admin,
            new_canister,
            "update_session_type",
            &SessionType::RegisteredSession,
        );
        match update_res {
            Ok(_) => (),
            Err(e) if e == "Session Already marked as Registered Session" => (),
            e => panic!("{e:?}"),
        };

        new_canister
    }

    pub fn game_balance(&self, individual_canister: Principal) -> BalanceInfo {
        execute_query(
            &self.pic,
            Principal::anonymous(),
            individual_canister,
            "pd_balance_info",
            &(),
        )
    }

    pub fn ledger_balance(&self, user: Principal) -> Nat {
        execute_query(
            &self.pic,
            Principal::anonymous(),
            self.known_principals[&KnownPrincipalType::CanisterIdCkBTCLedger],
            "icrc1_balance_of",
            &Account {
                owner: user,
                subaccount: None,
            },
        )
    }

    pub fn pumps_and_dumps(&self, individual_canister: Principal) -> PumpsAndDumps {
        execute_query(
            &self.pic,
            Principal::anonymous(),
            individual_canister,
            "pumps_and_dumps",
            &(),
        )
    }

    pub fn played_game_count(&self, individual_canister: Principal) -> usize {
        execute_query(
            &self.pic,
            Principal::anonymous(),
            individual_canister,
            "played_game_count",
            &(),
        )
    }

    pub fn reconcile_user_state(
        &self,
        individual_canister: Principal,
        state_diffs: &Vec<PumpNDumpStateDiff>,
    ) {
        let global_admin = Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap();
        execute_update::<_, Result<(), String>>(
            &self.pic,
            global_admin,
            individual_canister,
            "reconcile_user_state",
            state_diffs,
        )
        .unwrap();
    }

    pub fn net_earnings(&self, individual_canister: Principal) -> Nat {
        execute_query(
            &self.pic,
            Principal::anonymous(),
            individual_canister,
            "net_earnings",
            &(),
        )
    }

    pub fn update_pd_onboarding_reward_for_all_subnets(&self, new_reward: Nat) {
        let platform_admin = self.known_principals[&KnownPrincipalType::UserIdGlobalSuperAdmin];
        execute_update_no_res(
            &self.pic,
            platform_admin,
            self.known_principals[&KnownPrincipalType::CanisterIdPlatformOrchestrator],
            "update_pd_onboarding_reward_for_all_subnets",
            &new_reward,
        );
    }

    pub fn ckbtc_ledger_approve(&self, from: Principal, spender: Principal, amount: u64) {
        execute_update_no_res(
            &self.pic,
            from,
            self.known_principals[&KnownPrincipalType::CanisterIdCkBTCLedger],
            "icrc2_approve",
            &ApproveArgs {
                fee: None,
                memo: None,
                from_subaccount: None,
                created_at_time: None,
                amount: Nat::from(amount),
                expected_allowance: None,
                expires_at: None,
                spender: Account {
                    owner: spender,
                    subaccount: None,
                },
            },
        )
    }
}

#[test]
fn newly_registered_user_should_have_1000_gdollr() {
    let harness = PumpNDumpHarness::default();
    let alice = get_mock_user_alice_principal_id();
    let alice_canister = harness.provision_individual_canister(alice);

    let gdollr_bal = harness.game_balance(alice_canister).balance;

    assert_eq!(gdollr_bal, Nat::from(1e9 as u64));

    let withdrawable_bal = harness.game_balance(alice_canister).withdrawable;
    assert_eq!(withdrawable_bal, Nat::from(0u32));
}

#[test]
fn claim_sats_and_stake_sats_should_work() {
    let harness = PumpNDumpHarness::default();

    let alice = get_mock_user_alice_principal_id();
    let alice_canister = harness.provision_individual_canister(alice);

    let to_claim = 1e4 as u64;

    let past_bal = harness.ledger_balance(alice);

    let global_admin = Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap();

    harness.reconcile_user_state(
        alice_canister,
        &vec![PumpNDumpStateDiff::CreatorReward(
            (to_claim + LEDGER_FEE * 2) as u128,
        )],
    );

    let treasury = harness
        .known_principals[&KnownPrincipalType::UserIdCkBTCTreasury];
    let claim_amount = to_claim + LEDGER_FEE * 2;
    harness.ckbtc_ledger_approve(treasury, alice_canister, claim_amount + LEDGER_FEE);
    execute_update::<_, Result<(), String>>(
        &harness.pic,
        global_admin,
        alice_canister,
        "redeem_satoshis",
        &(claim_amount as u128),
    )
    .unwrap();

    let new_bal = harness.ledger_balance(alice);

    assert_eq!(
        new_bal - past_bal.clone(),
        to_claim.clone() + LEDGER_FEE * 2
    );

    let amount = to_claim + LEDGER_FEE;
    harness.ckbtc_ledger_approve(alice, alice_canister, amount); 
    let past_game_bal = harness.game_balance(alice_canister);

    execute_update::<_, Result<(), String>>(
        &harness.pic,
        alice,
        alice_canister,
        "stake_sats_for_cents",
        &(to_claim as u128),
    )
    .unwrap();

    let new_bal = harness.ledger_balance(alice);
    assert_eq!(new_bal, past_bal);

    let new_game_bal = harness.game_balance(alice_canister);
    assert_eq!(
        new_game_bal.balance - past_game_bal.balance,
        to_claim as u128
    );
    assert_eq!(
        new_game_bal.withdrawable - past_game_bal.withdrawable,
        to_claim as u128
    );
}

#[test]
fn reconcile_user_state_should_work() {
    let harness = PumpNDumpHarness::default();

    let alice = get_mock_user_alice_principal_id();
    let alice_canister = harness.provision_individual_canister(alice);

    // Test Addition
    let past_bal = harness.game_balance(alice_canister).balance;
    let past_pd = harness.pumps_and_dumps(alice_canister);
    let past_game_count = harness.played_game_count(alice_canister);
    let past_earnings = harness.net_earnings(alice_canister);
    let games = [
        ParticipatedGameInfo {
            pumps: 10,
            dumps: 10,
            reward: 50 * GDOLLR_TO_E8S as u128,
            token_root: Principal::anonymous(),
            game_direction: GameDirection::Pump,
        },
        ParticipatedGameInfo {
            pumps: 10,
            dumps: 10,
            reward: 50 * GDOLLR_TO_E8S as u128,
            token_root: Principal::anonymous(),
            game_direction: GameDirection::Dump,
        },
    ];
    let (pumps, dumps, mut total_reward) =
        games
            .iter()
            .fold((0u64, 0u64, Nat::from(0u32)), |acc, gdata| {
                (
                    acc.0 + gdata.pumps,
                    acc.1 + gdata.dumps,
                    acc.2 + gdata.reward.clone(),
                )
            });
    let earnings = total_reward.clone();
    total_reward -= (pumps + dumps) * GDOLLR_TO_E8S;
    let state_diffs: Vec<_> = games
        .into_iter()
        .map(PumpNDumpStateDiff::Participant)
        .collect();

    harness.reconcile_user_state(alice_canister, &state_diffs);

    let new_bal = harness.game_balance(alice_canister).balance;
    assert_eq!(new_bal.clone() - past_bal, total_reward);
    let new_pd = harness.pumps_and_dumps(alice_canister);
    let new_game_count = harness.played_game_count(alice_canister);
    assert_eq!(new_pd.pumps - past_pd.pumps, pumps as u128);
    assert_eq!(new_pd.dumps - past_pd.dumps, dumps as u128);
    assert_eq!(new_game_count - past_game_count, state_diffs.len());
    let new_earnings = harness.net_earnings(alice_canister);
    assert_eq!(new_earnings - past_earnings, earnings);

    // Test Deduction
    let past_bal = new_bal;
    let state_diffs = vec![PumpNDumpStateDiff::Participant(ParticipatedGameInfo {
        pumps: 1,
        dumps: 0,
        reward: 0 as u128,
        token_root: Principal::anonymous(),
        game_direction: GameDirection::Dump,
    })];
    let to_deduct = GDOLLR_TO_E8S;

    harness.reconcile_user_state(alice_canister, &state_diffs);
    let new_bal = harness.game_balance(alice_canister).balance;
    assert_eq!(past_bal - new_bal.clone(), to_deduct as u128);

    // Test Creator Reward
    let past_bal = new_bal;
    let to_add = 1e4 as u64;
    let state_diffs = vec![PumpNDumpStateDiff::CreatorReward(to_add as u128)];

    harness.reconcile_user_state(alice_canister, &state_diffs);
    let new_bal = harness.game_balance(alice_canister).balance;

    assert_eq!(new_bal - past_bal, to_add as u128);
}

#[test]
fn onboarding_reward_should_update() {
    let harness = PumpNDumpHarness::default();

    // 4000 DOLR
    let new_reward = Nat::from(4 * 1e9 as u64);
    harness.update_pd_onboarding_reward_for_all_subnets(new_reward.clone());

    let alice = get_mock_user_alice_principal_id();
    let alice_canister = harness.provision_individual_canister(alice);

    let bal = harness.game_balance(alice_canister);
    assert_eq!(bal.balance, new_reward);
    assert_eq!(bal.net_airdrop_reward, new_reward);
}
