#!/usr/bin/env bash

LEDGER_ACCOUNT_ID=$(dfx identity get-principal)

dfx deploy --specified-id 6rdgd-kyaaa-aaaaq-aaavq-cai dollr_mock_ledger --argument "
  (variant {
    Init = record {
      token_symbol = \"DOLLR\";
      token_name = \"DOLLR\";
      decimals = null;
      transfer_fee = 10_000;
      metadata = vec {};
      minting_account = record {
        owner = principal \"$LEDGER_ACCOUNT_ID\";
        subaccount = null;
      };
      initial_balances = vec {
        record {
          record {
            owner = principal \"$LEDGER_ACCOUNT_ID\";
            subaccount = null;
          };
          10_000_000_000;
        };
      };
      maximum_number_of_accounts = null;
      accounts_overflow_trim_quantity = null;
      fee_collector_account = null;
      max_memo_length = null;
      feature_flags = opt record {
        icrc2 = true;
      };
      archive_options = record {
        num_blocks_to_archive = 1000;
        max_transactions_per_response = null;
        trigger_threshold = 2000;
        more_controller_ids = null;
        max_message_size_bytes = null;
        cycles_for_archive_creation = opt 10000000000000;
        node_max_memory_size_bytes = null;
        controller_id = principal \"$LEDGER_ACCOUNT_ID\";
      };
    }
  })
"
