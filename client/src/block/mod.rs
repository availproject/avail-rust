pub mod calls;
pub mod encoded;
pub mod events;
pub mod extrinsic;
pub mod shared;
pub mod signed;

pub use calls::ExtrinsicCalls;
pub use encoded::{EncodedExtrinsic, EncodedExtrinsics, ExtrinsicsOpts, Metadata};
pub use events::{AllEvents, Event, Events};
pub use extrinsic::{Extrinsic, Extrinsics};
pub use signed::{SignedExtrinsic, SignedExtrinsics};

use crate::{Client, Error, block::shared::BlockContext};
use avail_rust_core::{
	AccountId, AvailHeader, BlockInfo, HashNumber,
	grandpa::GrandpaJustification,
	rpc::{self},
	types::{
		HashStringNumber,
		substrate::{PerDispatchClassWeight, Weight},
	},
};

/// High-level handle bound to a specific block id (height or hash).
pub struct Block {
	ctx: BlockContext,
}

impl Block {
	/// Constructs a view over the block identified by `block_id`.
	///
	/// # Parameters
	/// - `client`: RPC client used for follow-up queries.
	/// - `block_id`: Block number, hash, or string convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Block helper bound to the supplied identifier.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Block { ctx: BlockContext::new(client, block_id.into()) }
	}

	/// Returns a helper focused on signed extrinsics contained in this block.
	///
	/// # Returns
	/// - `SignedExtrinsics`: View that exposes signed extrinsics for this block.
	pub fn signed(&self) -> signed::SignedExtrinsics {
		signed::SignedExtrinsics::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Returns a helper for decoding extrinsics contained in this block.
	///
	/// # Returns
	/// - `Extrinsics`: View that decodes raw extrinsics into runtime calls.
	pub fn extrinsics(&self) -> extrinsic::Extrinsics {
		extrinsic::Extrinsics::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Returns a helper for retrieving encoded extrinsic payloads in this block.
	///
	/// # Returns
	/// - `EncodedExtrinsics`: View over encoded extrinsic payloads and metadata.
	pub fn encoded(&self) -> encoded::EncodedExtrinsics {
		encoded::EncodedExtrinsics::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Returns a helper for inspecting extrinsic call payloads.
	///
	/// # Returns
	/// - `ExtrinsicCalls`: View dedicated to decoding extrinsic calls.
	pub fn calls(&self) -> calls::ExtrinsicCalls {
		calls::ExtrinsicCalls::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Fetches raw extrinsic metadata for this block.
	///
	/// # Parameters
	/// - `opts`: Filters and encoding settings for the RPC request.
	///
	/// # Returns
	/// - `Ok(Vec<rpc::ExtrinsicInfo>)`: Raw extrinsics that matched the supplied options.
	/// - `Err(Error)`: Identifier decoding failed or the RPC call returned an error.
	///
	/// # Side Effects
	/// - Performs an RPC call which may retry according to the configured policy.
	pub async fn raw_extrinsics(&self, opts: rpc::ExtrinsicOpts) -> Result<Vec<rpc::ExtrinsicInfo>, Error> {
		let chain = self.ctx.chain();
		chain.system_fetch_extrinsics(self.ctx.block_id.clone(), opts).await
	}

	/// Returns an event helper scoped to this block.
	///
	/// # Returns
	/// - `Events`: View that fetches events emitted by the block.
	pub fn events(&self) -> events::Events {
		events::Events::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Overrides the retry behaviour for future RPC calls made through this helper.
	///
	/// # Parameters
	/// - `value`: `Some(true)` to force retries, `Some(false)` to disable retries, `None` to inherit the client default.
	///
	/// # Returns
	/// - `()`: The override is stored for subsequent operations.
	///
	/// # Side Effects
	/// - Updates the internal retry setting used by follow-up RPC calls.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.ctx.set_retry_on_error(value);
	}

	/// Fetches the GRANDPA justification associated with this block, if any.
	///
	/// # Returns
	/// - `Ok(Some(GrandpaJustification))`: The runtime provided a justification.
	/// - `Ok(None)`: No justification exists for the requested block.
	/// - `Err(Error)`: Resolving the block identifier or performing the RPC call failed.
	///
	/// # Side Effects
	/// - Performs RPC calls to resolve the block and download the justification.
	pub async fn justification(&self) -> Result<Option<GrandpaJustification>, Error> {
		let block_id: HashNumber = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let at = match block_id {
			HashNumber::Hash(h) => chain
				.block_height(h)
				.await?
				.ok_or(Error::Other("Failed to find block from the provided hash".into()))?,
			HashNumber::Number(n) => n,
		};

		chain.grandpa_block_justification(at).await.map_err(|e| e.into())
	}

	/// Reports whether this helper retries RPC failures.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}

	/// Retrieves the UNIX timestamp stored in this block's runtime `timestamp.set` extrinsic.
	///
	/// # Returns
	/// - `Ok(u64)`: Timestamp provided by the block's timestamp extrinsic.
	/// - `Err(Error)`: The timestamp extrinsic was missing or the RPC lookup failed.
	///
	/// # Side Effects
	/// - Fetches extrinsic data over RPC, honouring the retry configuration.
	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.encoded().timestamp().await
	}

	/// Fetches high-level metadata (number, hash, parent) for this block.
	///
	/// # Returns
	/// - `Ok(BlockInfo)`: Metadata describing the block.
	/// - `Err(Error)`: Resolving the block identifier or making the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn info(&self) -> Result<BlockInfo, Error> {
		let chain = self.ctx.chain();
		chain.block_info_from(self.ctx.block_id.clone()).await
	}

	/// Fetches the header associated with this block.
	///
	/// # Returns
	/// - `Ok(AvailHeader)`: Header returned by the node.
	/// - `Err(Error)`: Resolving the block identifier or performing the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		shared::header(&self.ctx.client, self.ctx.block_id.clone()).await
	}

	/// Fetches the author recorded for this block.
	///
	/// # Returns
	/// - `Ok(AccountId)`: Account identifier attributed as the block author.
	/// - `Err(Error)`: Resolving the block identifier or performing the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn author(&self) -> Result<AccountId, Error> {
		let chain = self.ctx.chain();
		chain.block_author(self.ctx.block_id.clone()).await
	}

	/// Counts how many extrinsics the block contains.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of extrinsics recorded in the block.
	/// - `Err(Error)`: Enumerating the extrinsics failed.
	///
	/// # Side Effects
	/// - Fetches extrinsic metadata over RPC, honouring the retry configuration.
	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.encoded().extrinsic_count().await
	}

	/// Counts how many events were emitted in this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of events exposed by the node.
	/// - `Err(Error)`: The RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn event_count(&self) -> Result<u32, Error> {
		let chain = self.ctx.chain();
		chain.block_event_count(self.ctx.block_id.clone()).await
	}

	/// Retrieves the dispatch-class weight totals reported for this block.
	///
	/// # Returns
	/// - `Ok(PerDispatchClassWeight)`: Weight data grouped by dispatch class.
	/// - `Err(Error)`: Fetching the weight via RPC failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		let chain = self.ctx.chain();
		chain.block_weight(self.ctx.block_id.clone()).await
	}

	/// Aggregates the weight consumed by extrinsics, based on success and failure events.
	///
	/// # Returns
	/// - `Ok(Weight)`: Sum of weights observed in extrinsic success or failure events.
	/// - `Err(Error)`: Fetching or decoding the event data failed.
	///
	/// # Side Effects
	/// - Fetches block events over RPC, honouring the retry configuration.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		self.events().extrinsic_weight().await
	}
}

#[cfg(test)]
pub mod test {
	use crate::{Client, MAINNET_ENDPOINT};

	#[tokio::test]
	pub async fn block_weight_test() {
		let client = Client::new(MAINNET_ENDPOINT).await.unwrap();
		let block = client.block(2042867);

		let extrinsic_weight = block.extrinsic_weight().await.unwrap();
		let block_weight = block.weight().await.unwrap();

		assert_eq!(extrinsic_weight.ref_time, 25047612000);
		assert_eq!(extrinsic_weight.proof_size, 1493);
		assert_eq!(block_weight.normal.ref_time, 0);
		assert_eq!(block_weight.normal.proof_size, 0);
		assert_eq!(block_weight.operational.ref_time, 0);
		assert_eq!(block_weight.operational.proof_size, 0);
		assert_eq!(block_weight.mandatory.ref_time, 28104773000);
		assert_eq!(block_weight.mandatory.proof_size, 116950);
	}

	#[tokio::test]
	pub async fn block_info_test() {
		let client = Client::new(MAINNET_ENDPOINT).await.unwrap();
		let block = client.block(2042867);

		let info = block.info().await.unwrap();

		assert_eq!(info.height, 2042867);
		assert_eq!(
			std::format!("{:?}", info.hash),
			"0x45c4fb5b83053dc5816eb0d532eba7dbd971921946dd56031937542291de5a7d"
		);
	}

	#[tokio::test]
	pub async fn block_event_count_test() {
		let client = Client::new(MAINNET_ENDPOINT).await.unwrap();
		let block = client.block(2042867);

		let count = block.event_count().await.unwrap();
		assert_eq!(count, 3);
	}

	#[tokio::test]
	pub async fn block_extrinsic_count_test() {
		let client = Client::new(MAINNET_ENDPOINT).await.unwrap();
		let block = client.block(2042867);

		let count = block.extrinsic_count().await.unwrap();
		assert_eq!(count, 2);
	}

	#[tokio::test]
	pub async fn block_author_test() {
		let client = Client::new(MAINNET_ENDPOINT).await.unwrap();
		let block = client.block(2042867);

		let author = block.author().await.unwrap();
		assert_eq!(author.to_string(), String::from("5HeP6FZoHcDJxGgF4TauP4yyZGfDTzZtGB28RHvxXjRSm6h6"));
	}

	#[tokio::test]
	pub async fn block_timestamp_test() {
		let client = Client::new(MAINNET_ENDPOINT).await.unwrap();
		let block = client.block(2042867);

		let timestamp = block.timestamp().await.unwrap();
		assert_eq!(timestamp, 1760954220001);
	}
}
