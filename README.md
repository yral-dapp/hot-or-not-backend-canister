# Hot or Not backend canister

# Migrate to serde steps

- [ ] add derived Serialize trait to individual user canister and check if it compiles with SPrincipal still present
- [ ] If it compiles, implement both the pre_upgrade method and run a local deploy to test it
- [ ] Then a post_upgrade to verify everything still worked
- [ ] IF the above fails, then try the longer route below
- [ ] migrate to new version of Canister DATA without stable memory types
- [ ] remove stable memory types from the code
- [ ] rename and reset to original naming for CANISTER_DATA variable and type
- [ ] remove all dependencies of stable memory and speedy
- [ ] pre_upgrade for serde
- [ ] deploy locally
- [ ] push changes to origin
- [ ] post_upgrade for serde
- [ ] enable 2 tests, starts with "when_backups_are_run..."
- [ ] deploy and commit locally
- [ ] push changes to origin
- [ ] run ic_deploy for penultimate commit
- [ ] run ic_deploy for latest commit
- [ ] merge to main branch skipping CI
