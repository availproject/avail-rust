build:
    just fmt
    ./scripts/build_sdk.sh
check:
    cargo check && just fmt
metadata-build:
    ./scripts/build_api.sh
fmt:
    cargo +nightly fmt
lint:
    cargo clippy
lint-fix:
    cargo clippy --fix
doc:
    cargo doc --open
test:
    cargo test -- --nocapture
podman:
    podman run -it --rm --network host docker.io/availj/avail:v2.3.4.0 --dev
docker:
    docker run -it --rm --network host docker.io/availj/avail:v2.3.4.0 --dev
