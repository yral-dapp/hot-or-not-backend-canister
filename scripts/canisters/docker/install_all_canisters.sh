#!/bin/bash

dfx start --background --artificial-delay 0

scripts/canisters/local_deploy/install_all_canisters.sh -s
