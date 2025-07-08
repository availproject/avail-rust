#!/bin/bash
set -e

cd ./examples

cd ./account && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./batch && rm -f  Cargo.lock && rm -f -r ./target && cd ./../.
cd ./subscriptions && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./full_metadata && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./parallel_transaction_submission && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./subxt_storage && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./transaction_submission && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./transaction_submission_with_exp && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./event_client && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./block_client && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./custom_transaction && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./custom_rpc_client && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./custom_storage && rm -f Cargo.lock && rm -f -r ./target && cd ./../
cd ./constants && rm -f Cargo.lock && rm -f -r ./target && cd ./../
cd ./estimating_fees && rm -f Cargo.lock && rm -f -r ./target && cd ./../
