build:
    ./build_sdk.sh
build-metadata:
    ./build_api.sh
fmt:
    cargo +nightly fmt
lint:
    cargo clippy
lint-fix:
    cargo clippy --fix
