#!/bin/bash
set -x
set -e

# SDK
cargo check
cargo check --no-default-features --features "native"
cargo check --no-default-features --features "native, tokio"
cargo check --no-default-features --features "native, smol"
cargo check --no-default-features --features "native, tracing"
cargo check --no-default-features --features "native, reqwest"
cargo check --no-default-features --features "native, reqwest, tracing, tokio"

rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, tokio"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, tracing"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, reqwest"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm, reqwest, tracing"
