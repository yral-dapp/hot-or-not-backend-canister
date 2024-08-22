

#### To check the status of the deployment

- check if the platform orchestrator performed the step to upgrade subnet canister with appropriate version: [Platform Orchestrator function](https://dashboard.internetcomputer.org/canister/74zq4-iqaaa-aaaam-ab53a-cai#get_subnet_last_upgrade_status)
- check the status of upgrade for individual canisters in subnet orchestrators and verify the version. Example for one of the subnet orchesrator: [Subnet Orchestrator function](https://dashboard.internetcomputer.org/canister/rimrc-piaaa-aaaao-aaljq-cai#get_index_details_last_upgrade_status)

To test the platform orchestrator, you can use the following command:

`dfx canister call --query 74zq4-iqaaa-aaaam-ab53a-cai get_subnet_last_upgrade_status --network=ic`
 the canister id (74zq4-iqaaa-aaaam-ab53a-cai) is taken from the canister_ids.json file