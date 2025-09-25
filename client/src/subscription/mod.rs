pub mod block;
pub mod extrinsic;
pub mod justification;
pub mod sub;

pub use block::{BlockHeaderSub, BlockSub, EventsSub, LegacyBlockSub};
pub use extrinsic::{ExtrinsicSub, RawExtrinsicSub, TransactionSub};
pub use justification::GrandpaJustificationSub;
pub use sub::Sub;

use crate::Client;

fn should_retry(client: &Client, value: Option<bool>) -> bool {
	value.unwrap_or(client.is_global_retries_enabled())
}
