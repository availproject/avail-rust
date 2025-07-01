# Note
This repo uses `mdbook` to generate documentation and `just` to run all the shell commands.
You can install both tools by running and executing `./install_dependencies.sh`. It should work as long as you have cargo installed.

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

# Release Strategy
This project uses [GitHub Flow](https://www.alexhyett.com/git-flow-github-flow/) to manage release and branches.

# Logging
You can enable logging by calling `SDK::enable_logging()` in your code and by using the `RUST_LOG` env variable. 
Example for just our logs:
```bash
RUST_LOG=info cargo run
```

Example for all Logs:
```bash
RUST_LOG=debug cargo run
```
