# SNS Local Testing Feedback

- Here's an additional feedback. For a multi canister architecture, it makes sense to make an upgrade proposal that includes all the canisters upgrades as part of a single proposal and have it be atomic. Otherwise, if some were to pass and some were to fail, it might break the canisters as they work as a coherent whole.
  We already have [something similar for registering dapp canisters](https://internetcomputer.org/docs/current/developer-docs/integrations/sns/get-sns/testflight#register-dapp-canisters-with-sns-root) where you can specify multiple canister IDs, so why not for upgrade proposals as well

- How do we simulate actual upgrade proposals using the mainnet-intended sns_init.yaml since we don't control all the identities and investors won't be able to vote on the proposals using a CLI based workflow? An additional pitfall is that sns controlled dapps in the pre-sale mode might get stuck with pre-sale upgrade if they don't have enough developer voting power to make the upgrade go through.
