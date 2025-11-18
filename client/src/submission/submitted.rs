//! Builders for submitting extrinsics and inspecting their on-chain lifecycle.

use crate::{
	Client, Error, UserError,
	block::{self, Block},
	conversions,
	subscription::Sub,
	transaction_options::{RefinedMortality, RefinedOptions},
};
use avail_rust_core::{
	AccountId, BlockInfo, EncodeSelector, H256, HasHeader, RpcError, rpc::ExtrinsicOpts,
	substrate::extrinsic::ExtrinsicAdditional, types::metadata::HashString,
};
use codec::Decode;
#[cfg(feature = "tracing")]
use tracing::info;

/// Handle to a transaction that has already been submitted to the network along with the contextual
/// information required to query its lifecycle.
#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub ext_hash: H256,
	pub account_id: AccountId,
	pub options: RefinedOptions,
	pub additional: ExtrinsicAdditional,
}

impl SubmittedTransaction {
	/// Creates a new submitted transaction handle using previously gathered metadata.
	///
	/// This does not perform any network calls; it simply stores the information needed to later
	/// resolve receipts or query status.
	pub fn new(
		client: Client,
		ext_hash: H256,
		account_id: AccountId,
		options: RefinedOptions,
		additional: ExtrinsicAdditional,
	) -> Self {
		Self { client, ext_hash, account_id, options, additional }
	}

	/// Produces a receipt describing how the transaction landed on chain, if it did at all.
	///
	/// # Returns
	/// - `Ok(Some(TransactionReceipt))` when the transaction is found in the searched block range.
	/// - `Ok(None)` when the transaction cannot be located within the mortality window implied by
	///   `options`.
	/// - `Err(Error)` when the underlying RPC or subscription queries fail.
	///
	/// Set `use_best_block` to `true` to follow the node's best chain (potentially including
	/// non-finalized blocks) or `false` to restrict the search to finalized blocks.
	pub async fn receipt(&self, use_best_block: bool) -> Result<Option<TransactionReceipt>, Error> {
		Utils::transaction_receipt(
			self.client.clone(),
			self.ext_hash,
			self.options.nonce,
			&self.account_id,
			&self.options.mortality,
			use_best_block,
		)
		.await
	}
}

/// Indicates what happened to a transaction after it was submitted.
///
/// The variants correspond to the states returned by the chain RPC when querying transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockState {
	/// The transaction was included in a block but the block may still be re-orged out.
	Included = 0,
	/// The block containing the transaction is finalized and immutable under normal circumstances.
	Finalized = 1,
	/// The transaction was seen but ended up discarded (e.g. due to invalidation).
	Discarded = 2,
	/// The transaction could not be found on chain.
	DoesNotExist = 3,
}

impl std::fmt::Display for BlockState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BlockState::Included => std::write!(f, "Included"),
			BlockState::Finalized => std::write!(f, "Finalized"),
			BlockState::Discarded => std::write!(f, "Discarded"),
			BlockState::DoesNotExist => std::write!(f, "DoesNotExist"),
		}
	}
}

/// Detailed information about where a transaction was found on chain.
#[derive(Clone)]
pub struct TransactionReceipt {
	client: Client,
	pub block_hash: H256,
	pub block_height: u32,
	pub ext_hash: H256,
	pub ext_index: u32,
}

impl TransactionReceipt {
	/// Wraps the provided block and transaction references without performing network IO.
	pub fn new(client: Client, block_hash: H256, block_height: u32, ext_hash: H256, ext_index: u32) -> Self {
		Self { client, block_hash, block_height, ext_hash, ext_index }
	}

	/// Returns the current lifecycle state of the containing block.
	///
	/// # Returns
	/// - `Ok(BlockState)` on success.
	/// - `Err(Error)` if the RPC request fails or the node cannot provide the block state.
	pub async fn block_state(&self) -> Result<BlockState, Error> {
		self.client.chain().block_state(self.block_hash).await
	}

	/// Fetches and decodes the extrinsic at the recorded index within the block.
	///
	/// # Returns
	/// - `Ok(Extrinsic<T>)` when the extrinsic exists and decodes as `T`.
	/// - `Err(Error)` when the extrinsic is missing, cannot be decoded as `T`, or RPC access fails.
	pub async fn extrinsic<T: HasHeader + Decode>(&self) -> Result<block::BlockExtrinsic<T>, Error> {
		let block = Block::new(self.client.clone(), self.block_hash).extrinsics();
		let ext: Option<block::BlockExtrinsic<T>> = block.get(self.ext_index).await?;
		let Some(ext) = ext else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(ext)
	}

	/// Returns the raw extrinsic bytes or a different encoding if requested.
	///
	/// # Returns
	/// - `Ok(EncodedExtrinsic)` with the requested encoding.
	/// - `Err(Error)` when the extrinsic cannot be found or an RPC failure occurs.
	pub async fn encoded(&self) -> Result<block::BlockEncodedExtrinsic, Error> {
		let block = Block::new(self.client.clone(), self.block_hash).encoded();
		let ext = block.get(self.ext_index).await?;
		let Some(ext) = ext else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(ext)
	}

	/// Fetches the events emitted as part of the transaction execution.
	///
	/// # Returns
	/// - `Ok(ExtrinsicEvents)` when the extrinsic exists and events are available.
	/// - `Err(Error)` when the events cannot be located or fetched.
	pub async fn events(&self) -> Result<crate::block::events::BlockEvents, Error> {
		let block = Block::new(self.client.clone(), self.block_hash).events();
		let events = block.extrinsic(self.ext_index).await?;
		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	/// Iterates block-by-block from `block_start` through `block_end` (inclusive) looking for an
	/// extrinsic whose hash matches `tx_hash`.
	///
	/// Returns `Ok(Some(TransactionReceipt))` as soon as a match is found, `Ok(None)` when the
	/// entire range has been exhausted without a match, and bubbles up any RPC or subscription
	/// errors encountered along the way.
	///
	/// Fails fast with a validation error when `block_start > block_end`. When `use_best_block`
	/// is `true`, the search follows the node's best chain; otherwise it restricts the iteration to
	/// finalized blocks only.
	pub async fn from_range(
		client: Client,
		ext_hash: impl Into<HashString>,
		block_start: u32,
		block_end: u32,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, Error> {
		if block_start > block_end {
			return Err(UserError::ValidationFailed("Block Start cannot start after Block End".into()).into());
		}

		let tx_hash = conversions::hash_string::to_hash(ext_hash)?;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(use_best_block);
		sub.set_block_height(block_start);

		loop {
			let block_info = sub.next().await?;

			let block = Block::new(client.clone(), block_info.height);
			let opts = ExtrinsicOpts::new().filter(tx_hash).encode_as(EncodeSelector::None);
			let infos = block.extrinsic_infos(opts).await?;

			if let Some(info) = infos.first() {
				let tr = TransactionReceipt::new(
					client.clone(),
					block_info.hash,
					block_info.height,
					info.ext_hash,
					info.ext_index,
				);
				return Ok(Some(tr));
			}

			if block_info.height >= block_end {
				return Ok(None);
			}
		}
	}
}

/// Convenience helpers for locating transactions on chain.
pub struct Utils;
impl Utils {
	/// Resolves the canonical receipt for a transaction if it landed on chain within its mortality window.
	///
	/// # Returns
	/// - `Ok(Some(TransactionReceipt))` when a matching inclusion is located.
	/// - `Ok(None)` when no matching transaction exists in the searched range.
	/// - `Err(Error)` when RPC queries fail or input validation detects an inconsistency.
	pub async fn transaction_receipt(
		client: Client,
		tx_hash: H256,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, Error> {
		let Some(block_info) =
			Self::find_correct_block_info(&client, nonce, tx_hash, account_id, mortality, use_best_block).await?
		else {
			return Ok(None);
		};

		let block = Block::new(client.clone(), block_info.hash);
		let opts = ExtrinsicOpts::new().filter(tx_hash).encode_as(EncodeSelector::None);
		let ext_info = block.extrinsic_infos(opts).await?;

		let Some(ext_info) = ext_info.first() else {
			return Ok(None);
		};

		Ok(Some(TransactionReceipt::new(
			client, block_info.hash, block_info.height, ext_info.ext_hash, ext_info.ext_index,
		)))
	}

	/// Inspects blocks following the transaction's mortality and returns the first matching inclusion.
	///
	/// The search starts at `mortality.block_height` and proceeds one block at a time until the
	/// mortality period expires, optionally following the node's best chain when `use_best_block` is
	/// `true`.
	///
	/// # Returns
	/// - `Ok(Some(BlockInfo))` once an inclusion is confirmed or a higher nonce proves execution.
	/// - `Ok(None)` when the mortality period elapses without finding a match.
	/// - `Err(Error)` if block streaming or nonce queries fail.
	pub async fn find_correct_block_info(
		client: &Client,
		nonce: u32,
		tx_hash: H256,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<BlockInfo>, Error> {
		let mortality_ends_height = mortality.block_height.saturating_add(mortality.period as u32);

		let mut sub = Sub::new(client.clone());
		sub.set_block_height(mortality.block_height);
		sub.use_best_block(use_best_block);

		let mut current_block_height = mortality.block_height;

		#[cfg(feature = "tracing")]
		{
			match use_best_block {
				true => {
					let info = client.best().block_info().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, info.height, mortality_ends_height);
				},
				false => {
					let info = client.finalized().block_info().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, info.height, mortality_ends_height);
				},
			};
		}

		while mortality_ends_height >= current_block_height {
			let info = sub.next().await?;
			current_block_height = info.height;

			let state_nonce = client.chain().block_nonce(account_id.clone(), info.hash).await?;
			if state_nonce > nonce {
				trace_new_block(nonce, state_nonce, account_id, info, true);
				return Ok(Some(info));
			}
			if state_nonce == 0 {
				let block = Block::new(client.clone(), info.hash);
				let opts = ExtrinsicOpts::new().filter(tx_hash).encode_as(EncodeSelector::None);
				let ext = block.extrinsic_infos(opts).await?;
				if !ext.is_empty() {
					trace_new_block(nonce, state_nonce, account_id, info, true);
					return Ok(Some(info));
				}
			}

			trace_new_block(nonce, state_nonce, account_id, info, false);
		}

		Ok(None)
	}
}

/// Emits optional tracing output detailing nonce progression while searching for a transaction.
///
/// When the `tracing` feature is disabled this function does nothing; otherwise it records each
/// inspected block along with whether the search completed.
fn trace_new_block(nonce: u32, state_nonce: u32, account_id: &AccountId, block_info: BlockInfo, search_done: bool) {
	#[cfg(feature = "tracing")]
	{
		if search_done {
			info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done", nonce, account_id, block_info.height, block_info.hash, state_nonce);
		} else {
			info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}.", nonce, account_id, block_info.height, block_info.hash, state_nonce);
		}
	}

	#[cfg(not(feature = "tracing"))]
	{
		let _ = (nonce, state_nonce, account_id, block_info, search_done);
	}
}
