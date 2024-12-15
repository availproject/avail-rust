build:
    ./build_sdk.sh
build-metadata:
    ./build_api.sh
build-book:
    ./build_book.sh
fmt:
    cargo +nightly fmt &&  cd ./examples && cargo +nightly fmt
lint:
    cargo clippy
lint-fix:
    cargo clippy --fix
serve-book:
    cd ./docs && mdbook serve
doc:
    cargo doc --open
examples:
    cd ./examples && RUST_LOG=debug cargo run

