build:
    ./build_sdk.sh
check:
    just build
metadata-build:
    ./build_api.sh
book-build:
    cd ./documentation && mdbook build
book-serve:
    cd ./documentation && mdbook serve
book-deploy:
    just book-build
    rm -rf ./docs
    mv ./documentation/book/html ./docs
fmt:
    cargo +nightly fmt &&  cd ./examples && cargo +nightly fmt
lint:
    cargo clippy
lint-fix:
    cargo clippy --fix
doc:
    cargo doc --open
examples:
    cd ./examples && RUST_LOG=debug cargo run
podman:
    podman run -it --rm --network host docker.io/availj/avail:v2.2.5.1 --dev
docker:
    docker run -it --rm --network host docker.io/availj/avail:v2.2.5.1 --dev
