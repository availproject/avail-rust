#!/bin/bash
set -x
set -e

cd ./account && cargo run && cd ./../.
cd ./batch && cargo run  && cd ./../.
cd ./block && cargo run  && cd ./../.
cd ./block_indexing && cargo run  && cd ./../.
cd ./full_metadata && cargo run  && cd ./../.
cd ./parallel_transaction_submission && cargo run  && cd ./../.
cd ./storage && cargo run  && cd ./../.
cd ./transaction_submission && cargo run  && cd ./../.
cd ./transaction_submission_with_exp && cargo run  && cd ./../.
cd ./event-client && cargo run  && cd ./../.
