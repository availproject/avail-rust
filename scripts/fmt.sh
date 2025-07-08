#!/bin/bash
set -e

cd ./examples

cd ./account && cargo +nightly fmt && cd ./../.
cd ./batch && cargo +nightly fmt  && cd ./../.
cd ./subscriptions && cargo +nightly fmt  && cd ./../.
cd ./full_metadata && cargo +nightly fmt  && cd ./../.
cd ./parallel_transaction_submission && cargo +nightly fmt  && cd ./../.
cd ./subxt_storage && cargo +nightly fmt  && cd ./../.
cd ./transaction_submission && cargo +nightly fmt  && cd ./../.
cd ./transaction_submission_with_exp && cargo +nightly fmt  && cd ./../.
cd ./event_client && cargo +nightly fmt  && cd ./../.
cd ./block_client && cargo +nightly fmt  && cd ./../.
cd ./custom_transaction && cargo +nightly fmt  && cd ./../.
cd ./custom_rpc_client && cargo +nightly fmt  && cd ./../.
cd ./custom_storage  && cargo +nightly fmt  && cd ./../.
cd ./constants && cargo +nightly fmt  && cd ./../.
cd ./estimating_fees && cargo +nightly fmt  && cd ./../.
