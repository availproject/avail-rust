use crate::{
	Client, Error, UserError,
	block::{self, Block, events::BlockEvents},
	conversions,
	error_ops::ErrorOperation,
	platform,
	subscription::sub::{BlockQueryMode, Sub, SubConfig},
};
use avail_rust_core::{DataFormat, H256, HasHeader, RpcError, types::metadata::HashString};
use codec::Decode;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct WaitOption {
	pub mode: BlockQueryMode,
	pub timeout: Duration,
}

impl WaitOption {
	pub fn new(mode: BlockQueryMode) -> Self {
		Self { mode, timeout: Duration::from_mins(3) }
	}

	pub fn timeout(mut self, value: Duration) -> Self {
		self.timeout = value;
		self
	}
}

impl From<BlockQueryMode> for WaitOption {
	fn from(value: BlockQueryMode) -> Self {
		Self::new(value)
	}
}

impl Default for WaitOption {
	fn default() -> Self {
		Self {
			mode: BlockQueryMode::Finalized,
			timeout: Duration::from_mins(3),
		}
	}
}

pub type SubmissionOutcome = (TransactionReceipt, BlockEvents);

/// Handle for a transaction that has already been submitted.
#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub ext_hash: H256,
	pub block_start: u32,
	pub block_end: u32,
}

impl SubmittedTransaction {
	/// Creates a submitted transaction handle from known metadata.
	pub fn new(client: Client, ext_hash: H256, block_start: u32, block_end: u32) -> Self {
		Self { client, ext_hash, block_start, block_end }
	}

	pub async fn find_receipt(&self, opts: impl Into<WaitOption>) -> Result<FindReceiptOutcome, Error> {
		find_receipt(self.client.clone(), self.ext_hash, self.block_start, Some(self.block_end), opts.into()).await
	}

	pub async fn receipt(&self, opts: impl Into<WaitOption>) -> Result<TransactionReceipt, Error> {
		match self.find_receipt(opts).await? {
			FindReceiptOutcome::Found(receipt) => Ok(receipt),
			FindReceiptOutcome::NotFound => Err(Error::not_found_with_op(
				ErrorOperation::SubmissionWaitForReceipt,
				"Transaction was not found in the search window",
			)),
			FindReceiptOutcome::TimedOut => Err(Error::Timeout(std::format!(
				"[op:{}] Transaction receipt search timed out",
				ErrorOperation::SubmissionWaitForReceipt
			))),
		}
	}

	pub async fn outcome(&self, opts: impl Into<WaitOption>) -> Result<SubmissionOutcome, Error> {
		let receipt = self.receipt(opts).await?;
		let events = receipt.events().await?;
		Ok((receipt, events))
	}
}

#[derive(Debug, Clone)]
pub enum FindReceiptOutcome {
	Found(TransactionReceipt),
	NotFound,
	TimedOut,
}

pub async fn find_receipt(
	client: Client,
	ext_hash: H256,
	from_block_height: u32,
	max_block_height: Option<u32>,
	opts: WaitOption,
) -> Result<FindReceiptOutcome, Error> {
	let future = find_receipt_inner(client, ext_hash, from_block_height, max_block_height, opts);
	match platform::timeout(opts.timeout, future).await {
		Ok(result) => result,
		Err(_) => Ok(FindReceiptOutcome::TimedOut),
	}
}

async fn find_receipt_inner(
	client: Client,
	ext_hash: H256,
	from_block_height: u32,
	max_block_height: Option<u32>,
	opts: WaitOption,
) -> Result<FindReceiptOutcome, Error> {
	let allow_list = Some(vec![ext_hash.into()]);
	let mut sub = client
		.subscribe()
		.raw()
		.from_height(from_block_height)
		.mode(opts.mode)
		.build()
		.await?;

	loop {
		let block_info = sub.next().await?;

		let exts = client
			.chain()
			.fetch_extrinsics(block_info.block_hash, allow_list.clone(), Default::default(), DataFormat::None)
			.await?;

		if let Some(ext) = exts.first() {
			let (ext_hash, ext_index) = (ext.ext_hash, ext.ext_index);
			let receipt = TransactionReceipt::new(
				client.clone(),
				block_info.block_hash,
				block_info.block_height,
				ext_hash,
				ext_index,
			);
			return Ok(FindReceiptOutcome::Found(receipt));
		}

		if let Some(max_height) = max_block_height
			&& block_info.block_height > max_height
		{
			return Ok(FindReceiptOutcome::NotFound);
		}
	}
}

/// Transaction lifecycle state on chain.
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

/// Location details for a transaction inclusion.
#[derive(Debug, Clone)]
pub struct TransactionReceipt {
	client: Client,
	pub block_hash: H256,
	pub block_height: u32,
	pub ext_hash: H256,
	pub ext_index: u32,
}

impl TransactionReceipt {
	/// Creates a receipt from known block/extrinsic coordinates.
	pub fn new(client: Client, block_hash: H256, block_height: u32, ext_hash: H256, ext_index: u32) -> Self {
		Self { client, block_hash, block_height, ext_hash, ext_index }
	}

	/// Returns the lifecycle state of the containing block.
	pub async fn block_state(&self) -> Result<BlockState, Error> {
		self.client.chain().block_state(self.block_hash).await
	}

	/// Fetches and decodes the recorded extrinsic as `T`.
	pub async fn extrinsic<T: HasHeader + Decode>(&self) -> Result<block::TypedExtrinsic<T>, Error> {
		let block = Block::new(self.client.clone(), self.block_hash).extrinsics();
		let ext: Option<block::TypedExtrinsic<T>> = block.get_as(self.ext_index).await?;
		let Some(ext) = ext else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(ext)
	}

	/// Fetches the recorded extrinsic in untyped/raw form.
	pub async fn untyped_extrinsic(&self) -> Result<block::UntypedExtrinsic, Error> {
		let block = Block::new(self.client.clone(), self.block_hash).extrinsics();
		let ext = block.get(self.ext_index).await?;
		let Some(ext) = ext else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(ext)
	}

	pub async fn timestamp(&self) -> Result<u64, Error> {
		let block = Block::new(self.client.clone(), self.block_hash);
		block.timestamp().await
	}

	/// Fetches events emitted by the recorded extrinsic.
	pub async fn events(&self) -> Result<crate::block::events::BlockEvents, Error> {
		let block = Block::new(self.client.clone(), self.block_hash).events();
		let events = block.extrinsic(self.ext_index).await?;
		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	/// Searches a block range (inclusive) for the given extrinsic hash.
	/// Returns `Ok(None)` when no match is found.
	pub async fn from_range(
		client: Client,
		ext_hash: impl Into<HashString>,
		block_start: u32,
		block_end: u32,
		mode: BlockQueryMode,
	) -> Result<Option<TransactionReceipt>, Error> {
		if block_start > block_end {
			return Err(UserError::ValidationFailed("Block Start cannot start after Block End".into()).into());
		}

		let tx_hash = conversions::hash_string::to_hash(ext_hash)?;
		let config = SubConfig { mode, start_height: Some(block_start), ..Default::default() };
		let mut sub = Sub::init(client.clone(), config).await.map_err(Error::from)?;

		loop {
			let block_info = sub.next().await?;

			let block = Block::new(client.clone(), block_info.height);
			let exts = block
				.extrinsics()
				.rpc(Some(vec![tx_hash.into()]), Default::default(), DataFormat::None)
				.await?;

			if let Some(info) = exts.first() {
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
