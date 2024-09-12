#!/bin/bash

dfx start --background --artificial-delay 0

scripts/canisters/local_deploy/setup_icp_ledger.sh
scripts/canisters/local_deploy/install_all_canisters.sh -s
scripts/canisters/docker/populate_posts.sh
