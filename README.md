# Note
This repo uses `mdbook` to generate documentation and `just` to run all the shell commands.
You can install both tools by running and executing `./install_dependencies.sh`. It should work as long as you have cargo installed.

Here is the list of available `just` commands (`just -l`):
- build - Builds the sdk with all features permutations. Run this once you have done all the changes
- build-book - Builds the book that contains documentation
- build-metadata - Builds the metadata. Run Node before running this command. 
- fmt - Formats the SDK using nightly cargo. Run this once you have done all the changes
- lint - Runs clippy
- lint-fix - Runs clippy --fix
- serve-book - Generates and serves the documentation at `http://localhost:3000`

# Release Strategy
This project uses [GitHub Flow](https://www.alexhyett.com/git-flow-github-flow/) to manage release and branches.

# Documentation
[Link](https://availproject.github.io/avail-rust/) to documentation (web preview of examples)


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
