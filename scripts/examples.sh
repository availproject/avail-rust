#!/bin/bash
set -e

cd ./examples

NAMES="./account ./batch ./client ./subscriptions ./parallel_transaction_submission ./transaction_submission ./transaction_submission_with_exp ./event_client ./block_client ./custom_transaction ./custom_rpc_client ./custom_storage ./constants ./estimating_fees ./storage ./encoding_decoding ./transaction_receipt"

OP="$1"
if [ "$OP" == "check" ]; then
    for name in $NAMES; do
        cd "$name" && cargo check && cd ./../.
    done
fi

if [ "$OP" == "run" ]; then
    for name in $NAMES; do
        cd "$name" && cargo run && cd ./../.
    done
fi

if [ "$OP" == "clean" ]; then
    for name in $NAMES; do
        cd "$name" && rm -f Cargo.lock && rm -f -r ./target && cd ./../.
    done
fi

if [ "$OP" == "fmt" ]; then
    for name in $NAMES; do
        cd "$name" && cargo +nightly fmt && cd ./../.
    done
fi
