//! Helpers for building block, extrinsic, and justification streaming subscriptions.

pub mod block;
pub mod extrinsic;
pub mod justification;
pub mod sub;

pub use block::{BlockEventsSub, BlockHeaderSub, BlockSub, LegacyBlockSub};
pub use extrinsic::{ExtrinsicSub, RawExtrinsicSub, TransactionSub};
pub use justification::GrandpaJustificationSub;
pub use sub::Sub;

use crate::Client;

/// Applies either the explicit override provided to a subscription or the client's global retry
/// configuration.
///
/// Returns `true` when RPC calls should be retried, `false` otherwise.
fn should_retry(client: &Client, value: Option<bool>) -> bool {
	value.unwrap_or(client.is_global_retries_enabled())
}
