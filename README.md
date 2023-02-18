# Hot or Not backend canister

# Migrate to serde steps

- [x] migrate to new version of Canister DATA without stable memory types
- [x] remove stable memory types from the code
- [x] rename and reset to original naming for CANISTER_DATA variable and type
- [x] remove all dependencies of stable memory and speedy
- [x] pre_upgrade for serde
- [x] deploy locally
- [x] push changes to origin
- [x] post_upgrade for serde
- [x] enable 2 tests, starts with "when_backups_are_run..."
- [x] deploy and commit locally
- [ ] run ic_deploy for penultimate commit
- [ ] push changes to origin
- [ ] run ic_deploy for latest commit
- [ ] merge to main branch skipping CI
