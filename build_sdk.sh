#!/bin/bash
set -x


cargo check
cargo check --no-default-features
cargo check --target wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --no-default-features

# Book
cd ./docs/book
cargo check
cargo check --no-default-features

# Extrinsics
cd ../extrinsics
cargo check
cargo check --no-default-features
