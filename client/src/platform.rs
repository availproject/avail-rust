//! Platform-specific async runtime primitives for native and WASM targets.
//!
//! Provides unified interfaces for `sleep` and `spawn` operations across different runtime environments.

#[cfg(feature = "native")]
pub use tokio::time::sleep;
#[cfg(feature = "native")]
pub use tokio::time::timeout;

#[cfg(feature = "wasm")]
pub use wasmtimer::tokio::sleep;
#[cfg(feature = "wasm")]
pub use wasmtimer::tokio::timeout;

#[cfg(feature = "native")]
pub use tokio::spawn;

#[cfg(feature = "wasm")]
pub use wasm_bindgen_futures::spawn_local as spawn;
