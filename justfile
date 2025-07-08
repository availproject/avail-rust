build:
    ./scripts/build_sdk.sh
check:
    cargo check && just fmt
metadata-build:
    ./scripts/build_api.sh
book-build:
    cd ./documentation && mdbook build
book-serve:
    cd ./documentation && mdbook serve
book-deploy:
    just book-build
    rm -rf ./docs
    mv ./documentation/book/html ./docs
book-publish:
    git diff --quiet
    git checkout gh-page
    git reset --hard main
    just book-deploy
    git add .
    git commit -m 'Book Deployed'
    git pf
fmt:
    cargo +nightly fmt
    just examples-fmt
lint:
    cargo clippy
lint-fix:
    cargo clippy --fix
doc:
    cargo doc --open
examples-run:
    RUST_LOG=info ./scripts/run.sh
examples-clean:
    ./scripts/clean.sh
examples-fmt:
    ./scripts/fmt.sh
examples-check:
    ./scripts/check.sh
test:
    cargo test
podman:
    podman run -it --rm --network host docker.io/availj/avail:v2.2.5.1 --dev
docker:
    docker run -it --rm --network host docker.io/availj/avail:v2.2.5.1 --dev
    
