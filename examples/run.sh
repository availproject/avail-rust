#!/bin/bash
set -e

cd ./account && cargo run && cd ./../.
cd ./batch && cargo run  && cd ./../.
cd ./subscriptions && cargo run  && cd ./../.
cd ./full_metadata && cargo run  && cd ./../.
cd ./parallel_transaction_submission && cargo run  && cd ./../.
cd ./storage && cargo run  && cd ./../.
cd ./transaction_submission && cargo run  && cd ./../.
cd ./transaction_submission_with_exp && cargo run  && cd ./../.
cd ./event_client && cargo run  && cd ./../.
cd ./block_client && cargo run  && cd ./../.
cd ./custom_transaction && cargo run  && cd ./../.
cd ./custom_rpc_client && cargo run  && cd ./../.
