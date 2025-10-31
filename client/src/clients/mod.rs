

//! RPC client implementations for different transport layers and testing scenarios.

pub mod online_client;

#[cfg(all(feature = "reqwest", any(test, feature = "mocks")))]
pub mod mock_client;

#[cfg(feature = "reqwest")]
pub mod reqwest_client;
pub use online_client::OnlineClient;
#[cfg(feature = "reqwest")]
pub use reqwest_client::ReqwestClient;
