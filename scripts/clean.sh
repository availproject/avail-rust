#!/bin/bash
set -e

cd ./examples

cd ./account && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./batch && rm -f  Cargo.lock && rm -f -r ./target && cd ./../.
cd ./subscriptions && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./full_metadata && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./parallel_transaction_submission && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./storage && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./transaction_submission && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./transaction_submission_with_exp && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./event_client && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./block_client && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./custom_transaction && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
cd ./custom_rpc_client && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
