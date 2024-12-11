# Note
This repo uses `mdbook` to generate documentation and `just` to run all the shell commands.
You can install both tools by running and executing `./install_dependencies.sh`. It should work as long as you have cargo installed.

Here is the list of available `just` commands (`just -l`):
- build - Builds the sdk with all features permutations. Run this once you have done all the changes
- build-book - Builds the book that contains documentation
- build-metadata - Builds the metadata. Run Node before running this command. Make sure to read `METADATA_NOTES.md`
- fmt - Formats the SDK using nightly cargo. Run this once you have done all the changes
- lint - Runs clippy
- lint-fix - Runs clippy --fix
- serve-book - Generates and serves the documentation at `http://localhost:3000`

