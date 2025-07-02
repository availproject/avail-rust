# For Library Users

**[Your documentation is in another castle.](https://github.com/availproject/avail-rust/blob/main/client/Cargo.toml)**

# For Library Developers

This repo uses `mdbook` to generate documentation and `just` to run all the
shell commands. You can install both tools by running and executing
`./install_dependencies.sh`. It should work as long as you have cargo installed.

Here is the list of available `just` commands (`just -l`):

- book-build
- book-deploy
- book-publish
- book-serve
- build
- check
- doc
- docker
- examples
- examples-clean
- fmt
- lint
- lint-fix
- metadata-build
- podman
- test

# Making sure that it builds for all targets

Once you have implemented and tested your changes, run `cargo clean` and
`just build`. This will clean all the existing build artifacts and build the
library using different feature flag permutations.

If `just` is not installed you can manually run `./build_sdk.sh` and it will do
the same.

## On New Node Release

Run `just metadata-build` or manually run `./build_api.sh` for the generated
metadata types to be regenerated.

# Release Strategy

This project uses [GitHub Flow](https://www.alexhyett.com/git-flow-github-flow/)
to manage release and branches.

# Contribution

Thank you for your interest in improving this project! As we are still adding
new features and finalizing existing ones, it would be helpful to first post
your idea in the
[discussions](https://github.com/availproject/avail-rust/discussions) or
[issues](https://github.com/availproject/avail-rust/issues).

Pull requests that only fix grammatical mistakes, resolve cargo clippy warnings,
or do not add any substantial value will be closed immediately without feedback.
