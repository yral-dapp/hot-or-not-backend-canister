#!/bin/bash

dfx start --clean --background --artificial-delay 0

scripts/canisters/local_deploy/install_all_canisters.sh -s