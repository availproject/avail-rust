#!/bin/bash
set -x
set -e

# SDK
cargo check
cargo check --no-default-features --features "native"
cargo check --no-default-features --features "native, reqwest"
cargo check --no-default-features --features "native, subxt"
cargo check --no-default-features --features "native, subxt_metadata"
cargo check --no-default-features --features "native, reqwest, subxt, subxt_metadata",

rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, reqwest"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, subxt"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, subxt_metadata"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, reqwest, subxt, subxt_metadata"

# Examples
# cd ./examples
# cargo check
# cargo check --no-default-features

