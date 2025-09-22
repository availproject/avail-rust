# Rust

The Avail Rust SDK requires Rust to be installed and configured.\
The recommended way to install Rust is with the official
[Rust installer](https://www.rust-lang.org/tools/install), which sets up
`rustup`, `cargo`, and the compiler.

## Ubuntu 24.04 (Empty Docker Image)

If you’re starting from a minimal image, install the required build tools first:

```bash
apt-get update
apt-get install -y build-essential curl libssl-dev pkg-config
```

Then install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
bash
```

Check that the installation worked:

```bash
rustc --version
cargo --version
```
