#!/bin/bash
set -e

cd ./examples

cd ./account && cargo check && cd ./../.
cd ./batch && cargo check  && cd ./../.
cd ./subscriptions && cargo check && cd ./../.
cd ./full_metadata && cargo check  && cd ./../.
cd ./parallel_transaction_submission && cargo check  && cd ./../.
cd ./subxt_storage && cargo check  && cd ./../.
cd ./transaction_submission && cargo check  && cd ./../.
cd ./transaction_submission_with_exp && cargo check  && cd ./../.
cd ./event_client && cargo check  && cd ./../.
cd ./block_client && cargo check  && cd ./../.
cd ./custom_transaction && cargo check && cd ./../.
cd ./custom_rpc_client && cargo check  && cd ./../.
cd ./custom_storage && cargo check  && cd ./../.
cd ./constants && cargo check  && cd ./../.
cd ./estimating_fees && cargo check  && cd ./../.
cd ./storage && cargo check  && cd ./../.
