## Testing

1. How to test changes locally in integration_tests?

There are some tests that are only run locally. e.g. `excessive_tokens_test::test_migrate_excessive_tokens`. 

To run these tests, follow the two step process. 

A. run the dfx build locally for the canister you have made changes to. (see README.md for that.)
B. run the command below
```
cargo test --package integration_tests --test upgrade --no-default-features -- excessive_tokens_test::test_migrate_excessive_tokens --exact --show-output
```

