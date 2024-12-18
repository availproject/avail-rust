build:
    ./build_sdk.sh
build-metadata:
    ./build_api.sh
build-book:
    cd ./docs && mdbook build
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
podman:
    podman run -it --rm --network host docker.io/availj/avail:v2.2.5.1 --dev
docker:
    docker run -it --rm --network host docker.io/availj/avail:v2.2.5.1 --dev
