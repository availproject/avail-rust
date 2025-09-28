# Feature Flags

This SDK comes with a few feature flags that let you choose how it runs —
whether you’re targeting native environments, WASM, or just want extra
integrations like HTTP or logging.

| Name    | Description                                                                                 | Default |
| ------- | ------------------------------------------------------------------------------------------- | ------- |
| native  | Add support for native environment. Typically used for desktop or server apps               | Yes     |
| wasm    | Add support for WebAssembly environments (browser, WASI, etc.)                              | No      |
| reqwest | Enables default HTTP client. Without this enabled, you would have to supply your own client | Yes     |
| tracing | Enables logging and tracing                                                                 | Yes     |

Typically, you’ll use either native or wasm, depending on your target.

A minimal example for native environment would look like:

```toml
avail-rust = { package = "avail-rust-client", version = "0.4.0-rc.1", default-features = false, features = ["native", "reqwest"] }
```
