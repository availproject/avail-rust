use crate::{
	Client, Error, UserError,
	block::{AllEvents, BlockEventsQuery, Metadata, extrinsic_options::Options, shared::BlockContext},
};
use avail_rust_core::{
	EncodeSelector, ExtrinsicDecodable, H256, HasHeader, RpcError,
	rpc::{self, ExtrinsicFilter},
	types::HashStringNumber,
};
use codec::Decode;

/// Detached view that decodes extrinsic call payloads within a block.
pub struct BlockExtrinsicCallsQuery {
	ctx: BlockContext,
}

impl BlockExtrinsicCallsQuery {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { ctx: BlockContext::new(client, block_id.into()) }
	}

	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<BlockExtrinsicCall<T>>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &BlockExtrinsicCallsQuery,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<BlockExtrinsicCall<T>>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			s.first::<T>(Options::new().filter(filter)).await
		}

		inner::<T>(self, extrinsic_id.into()).await
	}

	pub async fn first<T: HasHeader + Decode>(
		&self,
		mut opts: Options,
	) -> Result<Option<BlockExtrinsicCall<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let Some(call) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let call = T::from_call(call).map_err(Error::Other)?;
		let metadata = Metadata::new(first.ext_hash, first.ext_index, first.pallet_id, first.variant_id, block_id);

		Ok(Some(BlockExtrinsicCall::new(call, metadata)))
	}

	pub async fn last<T: HasHeader + Decode>(&self, mut opts: Options) -> Result<Option<BlockExtrinsicCall<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(last) = result.last_mut() else {
			return Ok(None);
		};

		let Some(call) = last.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let call = T::from_call(call).map_err(Error::Other)?;
		let metadata = Metadata::new(last.ext_hash, last.ext_index, last.pallet_id, last.variant_id, block_id);

		Ok(Some(BlockExtrinsicCall::new(call, metadata)))
	}

	pub async fn all<T: HasHeader + Decode>(&self, mut opts: Options) -> Result<Vec<BlockExtrinsicCall<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let extrinsics = chain.system_fetch_extrinsics(block_id, opts).await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for ext in extrinsics {
			let Some(call) = ext.data else {
				return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
			};

			let call = T::from_call(call).map_err(Error::Other)?;
			let metadata = Metadata::new(ext.ext_hash, ext.ext_index, ext.pallet_id, ext.variant_id, block_id);

			result.push(BlockExtrinsicCall::new(call, metadata));
		}

		Ok(result)
	}

	/// Counts how many extrinsic calls match the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to count.
	///
	/// # Returns
	/// - `Ok(usize)`: Number of matching extrinsics.
	/// - `Err(Error)`: The RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn count<T: HasHeader>(&self, opts: Options) -> Result<usize, Error> {
		let mut opts: rpc::ExtrinsicOpts = opts.to_rpc_opts(EncodeSelector::None);
		opts.transaction_filter = T::HEADER_INDEX.into();

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let result = chain.system_fetch_extrinsics(block_id, opts).await?;

		Ok(result.len())
	}

	/// Reports whether any extrinsic matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to test.
	///
	/// # Returns
	/// - `Ok(true)`: At least one matching extrinsic exists.
	/// - `Ok(false)`: No extrinsics matched the filters.
	/// - `Err(Error)`: The RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call via [`ExtrinsicCalls::count`] and may retry according to the retry policy.
	pub async fn exists<T: HasHeader>(&self, opts: Options) -> Result<bool, Error> {
		self.count::<T>(opts).await.map(|x| x > 0)
	}

	/// Overrides the retry behaviour for future extrinsic-call lookups.
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

	/// Reports whether call lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}
}

#[derive(Debug, Clone)]
pub struct BlockExtrinsicCall<T: HasHeader + Decode> {
	/// Decoded runtime call payload.
	pub call: T,
	/// Metadata describing where the extrinsic was found.
	pub metadata: Metadata,
}

impl<T: HasHeader + Decode> BlockExtrinsicCall<T> {
	pub fn new(call: T, metadata: Metadata) -> Self {
		Self { call, metadata }
	}

	/// Fetches events emitted by this transaction.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch event data.
	///
	/// # Returns
	/// - `Ok(AllEvents)`: Wrapper containing events for this extrinsic.
	/// - `Err(Error)`: Extrinsic emitted no events or the RPC request failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry according to the client's configuration.
	pub async fn events(&self, client: Client) -> Result<AllEvents, Error> {
		let events = BlockEventsQuery::new(client, self.metadata.block_id)
			.extrinsic(self.ext_index())
			.await?;
		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	/// Returns the index of this transaction inside the block.
	///
	/// # Returns
	/// - `u32`: Index of the extrinsic within the block.
	///
	/// # Side Effects
	/// - None; reads cached metadata.
	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	/// Returns the transaction hash.
	///
	/// # Returns
	/// - `H256`: Hash of the extrinsic.
	///
	/// # Side Effects
	/// - None; reads cached metadata.
	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}
}
