//! Convenience helpers for inspecting block data, extrinsics, and events via RPC.

use crate::{Client, Error, UserError};
use avail_rust_core::{
	AccountId, AvailHeader, BlockInfo, EncodeSelector, Extrinsic as CoreExtrinsic, ExtrinsicSignature, H256, HasHeader,
	HashNumber, MultiAddress, RpcError, TransactionEventDecodable, avail,
	grandpa::GrandpaJustification,
	rpc::{self, BlockPhaseEvent, ExtrinsicFilter, SignerPayload},
	types::{
		HashStringNumber, RuntimePhase,
		substrate::{PerDispatchClassWeight, Weight},
	},
};
use codec::Decode;

/// High-level handle bound to a specific block id (height or hash).
pub struct Block {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
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
		Block { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Returns a helper focused on signed extrinsics contained in this block.
	///
	/// # Returns
	/// - `SignedExtrinsics`: View that exposes signed extrinsics for this block.
	pub fn signed(&self) -> SignedExtrinsics {
		SignedExtrinsics::new(self.client.clone(), self.block_id.clone())
	}

	/// Returns a helper for decoding extrinsics contained in this block.
	///
	/// # Returns
	/// - `Extrinsics`: View that decodes raw extrinsics into runtime calls.
	pub fn extrinsics(&self) -> Extrinsics {
		Extrinsics::new(self.client.clone(), self.block_id.clone())
	}

	/// Returns a helper for retrieving encoded extrinsic payloads in this block.
	///
	/// # Returns
	/// - `EncodedExtrinsics`: View over encoded extrinsic payloads and metadata.
	pub fn encoded(&self) -> EncodedExtrinsics {
		EncodedExtrinsics::new(self.client.clone(), self.block_id.clone())
	}

	/// Returns a helper for inspecting extrinsic call payloads.
	///
	/// # Returns
	/// - `ExtrinsicCalls`: View dedicated to decoding extrinsic calls.
	pub fn calls(&self) -> ExtrinsicCalls {
		ExtrinsicCalls::new(self.client.clone(), self.block_id.clone())
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
		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.system_fetch_extrinsics(block_id, opts).await
	}

	/// Returns an event helper scoped to this block.
	///
	/// # Returns
	/// - `Events`: View that fetches events emitted by the block.
	pub fn events(&self) -> Events {
		Events::new(self.client.clone(), self.block_id.clone())
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
		self.retry_on_error = value;
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
		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let at = match block_id {
			HashNumber::Hash(h) => self
				.client
				.chain()
				.retry_on(self.retry_on_error, None)
				.block_height(h)
				.await?
				.ok_or(Error::Other("Failed to find block from the provided hash".into()))?,
			HashNumber::Number(n) => n,
		};

		self.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.grandpa_block_justification(at)
			.await
			.map_err(|e| e.into())
	}

	/// Reports whether this helper retries RPC failures.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
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
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_info_from(self.block_id.clone()).await
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
		shared::header(&self.client, self.block_id.clone()).await
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
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_author(self.block_id.clone()).await
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
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_event_count(self.block_id.clone()).await
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
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_weight(self.block_id.clone()).await
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

/// Detached view that decodes extrinsic call payloads within a block.
pub struct ExtrinsicCalls {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl ExtrinsicCalls {
	/// Builds a helper dedicated to decoded extrinsic calls.
	///
	/// # Parameters
	/// - `client`: RPC client used to query extrinsic data.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Call-focused helper scoped to the target block.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Fetches a specific extrinsic call by hash, index, or string identifier.
	///
	/// # Parameters
	/// - `extrinsic_id`: Identifier used to select the extrinsic.
	///
	/// # Returns
	/// - `Ok(Some(T))`: The extrinsic matched the identifier and decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the identifier.
	/// - `Err(Error)`: Identifier decoding or the RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<T>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &ExtrinsicCalls,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<T>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			s.first::<T>(ExtrinsicsOpts::new().filter(filter)).await
		}

		inner::<T>(self, extrinsic_id.into()).await
	}

	/// Fetches the first extrinsic call that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to return.
	///
	/// # Returns
	/// - `Ok(Some(T))`: First matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: The RPC request or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn first<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Option<T>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let Some(data) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let mut iter = data.as_bytes();
		let data = T::decode(&mut iter)?;

		Ok(Some(data))
	}

	/// Fetches the last extrinsic call that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to return.
	///
	/// # Returns
	/// - `Ok(Some(T))`: Final matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: The RPC request or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn last<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Option<T>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.last_mut() else {
			return Ok(None);
		};

		let Some(data) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let mut iter = data.as_bytes();
		let data = T::decode(&mut iter)?;

		Ok(Some(data))
	}

	/// Fetches every extrinsic call that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to collect.
	///
	/// # Returns
	/// - `Ok(Vec<T>)`: Zero or more decoded extrinsic calls.
	/// - `Err(Error)`: The RPC request or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs an RPC call, decoding each payload, and may retry according to the retry policy.
	pub async fn all<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Vec<T>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let extrinsics = chain.system_fetch_extrinsics(block_id, opts).await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for ext in extrinsics {
			let Some(data) = ext.data else {
				return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
			};

			let mut iter = data.as_bytes();
			let data = T::decode(&mut iter)?;

			result.push(data);
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
	pub async fn count<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		let mut opts: rpc::ExtrinsicOpts = opts.to_rpc_opts(EncodeSelector::None);
		opts.transaction_filter = T::HEADER_INDEX.into();

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
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
	pub async fn exists<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
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
		self.retry_on_error = value;
	}

    /// Reports whether call lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
    pub fn should_retry_on_error(&self) -> bool {
        self.retry_on_error
            .unwrap_or_else(|| self.client.is_global_retries_enabled())
	}
}

/// View of block extrinsics as raw payloads with associated metadata.
pub struct EncodedExtrinsics {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl EncodedExtrinsics {
	/// Builds a raw extrinsic view for the specified block.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch encoded extrinsics.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Encoded-extrinsic helper scoped to the provided block.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Fetches a specific extrinsic and returns it in encoded form.
	///
	/// # Parameters
	/// - `extrinsic_id`: Hash, index, or string identifying the extrinsic to fetch.
	///
	/// # Returns
	/// - `Ok(Some(EncodedExtrinsic))`: Matching encoded extrinsic with metadata.
	/// - `Ok(None)`: No extrinsic matched the identifier.
	/// - `Err(Error)`: Identifier decoding or the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn get(&self, extrinsic_id: impl Into<HashStringNumber>) -> Result<Option<EncodedExtrinsic>, Error> {
		async fn inner(
			s: &EncodedExtrinsics,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<EncodedExtrinsic>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			let opts = ExtrinsicsOpts::new().filter(filter);

			s.first(opts).await
		}

		inner(self, extrinsic_id.into()).await
	}

	/// Returns the first encoded extrinsic matching the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to fetch.
	///
	/// # Returns
	/// - `Ok(Some(EncodedExtrinsic))`: First matching extrinsic with metadata and payload.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: Resolving the block identifier, performing the RPC call, or retrieving the payload failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn first(&self, opts: ExtrinsicsOpts) -> Result<Option<EncodedExtrinsic>, Error> {
		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let Some(data) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let metadata = Metadata::new(first.ext_hash, first.ext_index, first.pallet_id, first.variant_id, block_id);
		let ext = EncodedExtrinsic::new(data, metadata, first.signer_payload.take());

		Ok(Some(ext))
	}

	/// Returns the last encoded extrinsic matching the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to fetch.
	///
	/// # Returns
	/// - `Ok(Some(EncodedExtrinsic))`: Final matching extrinsic with metadata and payload.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: Resolving the block identifier, performing the RPC call, or retrieving the payload failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn last(&self, opts: ExtrinsicsOpts) -> Result<Option<EncodedExtrinsic>, Error> {
		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(last) = result.last_mut() else {
			return Ok(None);
		};

		let Some(data) = last.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let metadata = Metadata::new(last.ext_hash, last.ext_index, last.pallet_id, last.variant_id, block_id);
		let ext = EncodedExtrinsic::new(data, metadata, last.signer_payload.take());

		Ok(Some(ext))
	}

	/// Returns all encoded extrinsics matching the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to fetch.
	///
	/// # Returns
	/// - `Ok(Vec<EncodedExtrinsic>)`: Zero or more matching extrinsics.
	/// - `Err(Error)`: Resolving the block identifier, performing the RPC call, or retrieving a payload failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn all(&self, opts: ExtrinsicsOpts) -> Result<Vec<EncodedExtrinsic>, Error> {
		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let extrinsics = chain.system_fetch_extrinsics(block_id, opts).await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for ext in extrinsics {
			let metadata = Metadata::new(ext.ext_hash, ext.ext_index, ext.pallet_id, ext.variant_id, block_id);
			let Some(data) = ext.data else {
				return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
			};

			let enc_ext = EncodedExtrinsic::new(data, metadata, ext.signer_payload);
			result.push(enc_ext);
		}

		Ok(result)
	}

	/// Counts matching extrinsics without downloading their payloads.
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
	pub async fn count(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		let opts: rpc::ExtrinsicOpts = opts.to_rpc_opts(EncodeSelector::None);

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let result = chain.system_fetch_extrinsics(block_id, opts).await?;

		Ok(result.len())
	}

	/// Reports whether any encoded extrinsic matches the supplied filters.
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
	/// - Performs an RPC call via [`EncodedExtrinsics::count`] and may retry according to the retry policy.
	pub async fn exists(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
		self.count(opts).await.map(|x| x > 0)
	}

	/// Overrides the retry behaviour for future encoded-extrinsic lookups.
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
		self.retry_on_error = value;
	}

	/// Reports whether encoded-extrinsic lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

	/// Retrieves the UNIX timestamp recorded in this block's `timestamp.set` extrinsic.
	///
	/// # Returns
	/// - `Ok(u64)`: Timestamp provided by the block's timestamp extrinsic.
	/// - `Err(Error)`: The timestamp extrinsic was missing or the RPC lookup failed.
	///
	/// # Side Effects
	/// - Fetches extrinsic data over RPC, honouring the retry configuration.
	pub async fn timestamp(&self) -> Result<u64, Error> {
		let calls = ExtrinsicCalls::new(self.client.clone(), self.block_id.clone());

		let timestamp = calls.first::<avail::timestamp::tx::Set>(Default::default()).await?;
		let Some(timestamp) = timestamp else {
			return Err(Error::User(UserError::Other(std::format!(
				"No timestamp transaction found in block: {:?}",
				self.block_id
			))));
		};

		Ok(timestamp.now)
	}

	/// Fetches block metadata such as number and hash.
	///
	/// # Returns
	/// - `Ok(BlockInfo)`: Metadata describing the block.
	/// - `Err(Error)`: Resolving the block identifier or performing the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn info(&self) -> Result<BlockInfo, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_info_from(self.block_id.clone()).await
	}

	/// Fetches the block header.
	///
	/// # Returns
	/// - `Ok(AvailHeader)`: Header returned by the node.
	/// - `Err(Error)`: Resolving the block identifier or performing the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		shared::header(&self.client, self.block_id.clone()).await
	}

	/// Fetches the author attributed to this block.
	///
	/// # Returns
	/// - `Ok(AccountId)`: Account identified by the node as the block author.
	/// - `Err(Error)`: Resolving the block identifier or performing the RPC call failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn author(&self) -> Result<AccountId, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_author(self.block_id.clone()).await
	}

	/// Counts extrinsics included in this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of extrinsics recorded in the block.
	/// - `Err(Error)`: Extrinsic enumeration failed.
	///
	/// # Side Effects
	/// - Queries encoded extrinsics via RPC and may retry as configured.
	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.count(ExtrinsicsOpts::new()).await.map(|x| x as u32)
	}

	/// Counts events emitted by this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of events emitted by the block.
	/// - `Err(Error)`: RPC request failed.
	///
	/// # Side Effects
	/// - Issues an RPC request to the chain API and may retry as configured.
	pub async fn event_count(&self) -> Result<u32, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_event_count(self.block_id.clone()).await
	}

	/// Retrieves the block weight grouped by dispatch class.
	///
	/// # Returns
	/// - `Ok(PerDispatchClassWeight)`: Weight totals reported by the runtime.
	/// - `Err(Error)`: RPC request failed.
	///
	/// # Side Effects
	/// - Issues an RPC request and may retry as configured.
	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_weight(self.block_id.clone()).await
	}

	/// Aggregates the weight consumed by extrinsics in this block.
	///
	/// # Returns
	/// - `Ok(Weight)`: Total weight indicated by success and failure events.
	/// - `Err(Error)`: Event retrieval or decoding failed.
	///
	/// # Side Effects
	/// - Fetches block events via RPC and may retry as configured.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		let mut events = Events::new(self.client.clone(), self.block_id.clone());
		events.set_retry_on_error(self.retry_on_error);
		events.extrinsic_weight().await
	}
}

/// View of block extrinsics decoded into calls and optional signatures.
pub struct Extrinsics {
	xt: EncodedExtrinsics,
}

impl Extrinsics {
	/// Builds a decoded extrinsic view for the specified block.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch encoded extrinsics before decoding.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Helper that decodes extrinsics on demand.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { xt: EncodedExtrinsics::new(client, block_id) }
	}

	/// Fetches a specific extrinsic by hash, index, or string identifier.
	///
	/// # Parameters
	/// - `extrinsic_id`: Identifier used to select the extrinsic.
	///
	/// # Returns
	/// - `Ok(Some(Extrinsic<T>))`: Matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic matched the identifier.
	/// - `Err(Error)`: Identifier decoding, the RPC call, or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the encoded-extrinsic helper and may retry according to the retry policy.
	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<Extrinsic<T>>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &Extrinsics,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<Extrinsic<T>>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			s.first::<T>(ExtrinsicsOpts::new().filter(filter)).await
		}

		inner::<T>(self, extrinsic_id.into()).await
	}

	/// Returns the first extrinsic that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to fetch.
	///
	/// # Returns
	/// - `Ok(Some(Extrinsic<T>))`: First matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: The RPC call or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the encoded-extrinsic helper and may retry according to the retry policy.
	pub async fn first<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Option<Extrinsic<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let first = self.xt.first(opts).await?;
		let Some(first) = first else {
			return Ok(None);
		};

		let ext = CoreExtrinsic::<T>::try_from(first.data.as_str()).map_err(UserError::Decoding)?;
		let ext = Extrinsic::new(ext.signature, ext.call, first.metadata);

		Ok(Some(ext))
	}

	/// Returns the last extrinsic that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to fetch.
	///
	/// # Returns
	/// - `Ok(Some(Extrinsic<T>))`: Final matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: The RPC call or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the encoded-extrinsic helper and may retry according to the retry policy.
	pub async fn last<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Option<Extrinsic<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let last = self.xt.last(opts).await?;
		let Some(last) = last else {
			return Ok(None);
		};

		let ext = CoreExtrinsic::<T>::try_from(last.data.as_str()).map_err(UserError::Decoding)?;
		let ext = Extrinsic::new(ext.signature, ext.call, last.metadata);
		Ok(Some(ext))
	}

	/// Collects every extrinsic that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to decode.
	///
	/// # Returns
	/// - `Ok(Vec<Extrinsic<T>>)`: Zero or more matching extrinsics decoded as `T`.
	/// - `Err(Error)`: The RPC call or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the encoded-extrinsic helper and may retry according to the retry policy.
	pub async fn all<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Vec<Extrinsic<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let all = self.xt.all(opts).await?;
		let mut result = Vec::with_capacity(all.len());
		for raw_ext in all {
			let ext = CoreExtrinsic::<T>::try_from(raw_ext.data.as_str()).map_err(UserError::Decoding)?;
			let ext = Extrinsic::new(ext.signature, ext.call, raw_ext.metadata);
			result.push(ext);
		}

		Ok(result)
	}

	/// Counts matching extrinsics without decoding their payloads.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to count.
	///
	/// # Returns
	/// - `Ok(usize)`: Number of matching extrinsics.
	/// - `Err(Error)`: The RPC call failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the encoded-extrinsic helper and may retry according to the retry policy.
	pub async fn count<T: HasHeader>(&self, mut opts: ExtrinsicsOpts) -> Result<usize, Error> {
		opts.filter = Some(T::HEADER_INDEX.into());

		return self.xt.count(opts).await;
	}

	/// Reports whether any extrinsic matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to test.
	///
	/// # Returns
	/// - `Ok(true)`: At least one matching extrinsic exists.
	/// - `Ok(false)`: No extrinsics matched the filters.
	/// - `Err(Error)`: The RPC call failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the encoded-extrinsic helper and may retry according to the retry policy.
	pub async fn exists<T: HasHeader>(&self, mut opts: ExtrinsicsOpts) -> Result<bool, Error> {
		opts.filter = Some(T::HEADER_INDEX.into());

		return self.xt.exists(opts).await;
	}

	/// Overrides the retry behaviour for future decoded-extrinsic lookups.
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
		self.xt.set_retry_on_error(value);
	}

	/// Reports whether decoded-extrinsic lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.xt.should_retry_on_error()
	}

	/// Retrieves the UNIX timestamp recorded in this block.
	///
	/// # Returns
	/// - `Ok(u64)`: Timestamp from the runtime `timestamp.set` extrinsic.
	/// - `Err(Error)`: Timestamp extrinsic missing or RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.xt.timestamp().await
	}

	/// Fetches block metadata such as numbers and hashes.
	///
	/// # Returns
	/// - `Ok(BlockInfo)`: Metadata for the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn info(&self) -> Result<BlockInfo, Error> {
		self.xt.info().await
	}

	/// Fetches the block header.
	///
	/// # Returns
	/// - `Ok(AvailHeader)`: Header for the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		self.xt.header().await
	}

	/// Fetches the author attributed to this block.
	///
	/// # Returns
	/// - `Ok(AccountId)`: Account identified as the block author.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn author(&self) -> Result<AccountId, Error> {
		self.xt.author().await
	}

	/// Counts extrinsics included in this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of extrinsics recorded in the block.
	/// - `Err(Error)`: Extrinsic enumeration failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.xt.extrinsic_count().await
	}

	/// Counts events emitted by this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of events emitted in the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn event_count(&self) -> Result<u32, Error> {
		self.xt.event_count().await
	}

	/// Retrieves the block weight grouped by dispatch class.
	///
	/// # Returns
	/// - `Ok(PerDispatchClassWeight)`: Weight totals reported by the runtime.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		self.xt.weight().await
	}

	/// Aggregates the weight consumed by extrinsics in this block.
	///
	/// # Returns
	/// - `Ok(Weight)`: Total weight indicated by success and failure events.
	/// - `Err(Error)`: Event retrieval or decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the encoded extrinsics helper and may retry as configured.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		self.xt.extrinsic_weight().await
	}
}

/// View of block extrinsics restricted to signed transactions.
pub struct SignedExtrinsics {
	xt: Extrinsics,
}

impl SignedExtrinsics {
	/// Builds a signed-transaction view for the specified block.
	///
	/// # Parameters
	/// - `client`: RPC client used to access extrinsic data.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Helper that only surfaces signed extrinsics.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { xt: Extrinsics::new(client, block_id) }
	}

	/// Fetches a signed transaction by hash, index, or string identifier.
	///
	/// # Parameters
	/// - `extrinsic_id`: Identifier used to select the extrinsic.
	///
	/// # Returns
	/// - `Ok(Some(SignedExtrinsic<T>))`: Matching extrinsic decoded as `T` with a signature.
	/// - `Ok(None)`: No extrinsic matched the identifier.
	/// - `Err(Error)`: The extrinsic was unsigned, or the RPC call/decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<SignedExtrinsic<T>>, Error> {
		let ext = self.xt.get(extrinsic_id).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(
				UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into()
			);
		};

		let ext = SignedExtrinsic::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	/// Returns the first signed extrinsic that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which signed extrinsics to fetch.
	///
	/// # Returns
	/// - `Ok(Some(SignedExtrinsic<T>))`: First matching signed extrinsic decoded as `T`.
	/// - `Ok(None)`: No signed extrinsic satisfied the filters.
	/// - `Err(Error)`: The extrinsic was unsigned, or the RPC call/decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
	pub async fn first<T: HasHeader + Decode>(
		&self,
		opts: ExtrinsicsOpts,
	) -> Result<Option<SignedExtrinsic<T>>, Error> {
		let ext = self.xt.first(opts).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(
				UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into()
			);
		};

		let ext = SignedExtrinsic::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	/// Returns the last signed extrinsic that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which signed extrinsics to fetch.
	///
	/// # Returns
	/// - `Ok(Some(SignedExtrinsic<T>))`: Final matching signed extrinsic decoded as `T`.
	/// - `Ok(None)`: No signed extrinsic satisfied the filters.
	/// - `Err(Error)`: The extrinsic was unsigned, or the RPC call/decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
	pub async fn last<T: HasHeader + Decode>(&self, opts: ExtrinsicsOpts) -> Result<Option<SignedExtrinsic<T>>, Error> {
		let ext = self.xt.last(opts).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(
				UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into()
			);
		};

		let ext = SignedExtrinsic::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	/// Collects every signed extrinsic that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which signed extrinsics to fetch.
	///
	/// # Returns
	/// - `Ok(Vec<SignedExtrinsic<T>>)`: Zero or more signed extrinsics decoded as `T`.
	/// - `Err(Error)`: An extrinsic lacked a signature, or the RPC call/decoding failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
	pub async fn all<T: HasHeader + Decode>(&self, opts: ExtrinsicsOpts) -> Result<Vec<SignedExtrinsic<T>>, Error> {
		let all = self.xt.all::<T>(opts).await?;
		let mut result = Vec::with_capacity(all.len());
		for ext in all {
			let Some(signature) = ext.signature else {
				return Err(UserError::Other(
					"Extrinsic is unsigned; cannot decode it as a signed transaction.".into(),
				)
				.into());
			};
			result.push(SignedExtrinsic::new(signature, ext.call, ext.metadata));
		}

		Ok(result)
	}

	/// Counts matching signed extrinsics.
	///
	/// # Parameters
	/// - `opts`: Filters describing which signed extrinsics to count.
	///
	/// # Returns
	/// - `Ok(usize)`: Number of matching signed extrinsics.
	/// - `Err(Error)`: The RPC call failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
	pub async fn count<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		self.xt.count::<T>(opts).await
	}

	/// Reports whether any signed extrinsic matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which signed extrinsics to test.
	///
	/// # Returns
	/// - `Ok(true)`: At least one matching signed extrinsic exists.
	/// - `Ok(false)`: No signed extrinsics matched the filters.
	/// - `Err(Error)`: The RPC call failed.
	///
	/// # Side Effects
	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
	pub async fn exists<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
		self.xt.exists::<T>(opts).await
	}

	/// Overrides the retry behaviour for future signed-transaction lookups.
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
		self.xt.set_retry_on_error(value);
	}

	/// Reports whether signed-transaction lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.xt.should_retry_on_error()
	}

	/// Retrieves the UNIX timestamp recorded in this block.
	///
	/// # Returns
	/// - `Ok(u64)`: Timestamp from the runtime `timestamp.set` extrinsic.
	/// - `Err(Error)`: Timestamp extrinsic missing or RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.xt.timestamp().await
	}

	/// Fetches block metadata such as numbers and hashes.
	///
	/// # Returns
	/// - `Ok(BlockInfo)`: Metadata for the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn info(&self) -> Result<BlockInfo, Error> {
		self.xt.info().await
	}

	/// Fetches the block header.
	///
	/// # Returns
	/// - `Ok(AvailHeader)`: Header for the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		self.xt.header().await
	}

	/// Fetches the author attributed to this block.
	///
	/// # Returns
	/// - `Ok(AccountId)`: Account identified as the block author.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn author(&self) -> Result<AccountId, Error> {
		self.xt.author().await
	}

	/// Counts extrinsics included in this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of extrinsics recorded in the block.
	/// - `Err(Error)`: Extrinsic enumeration failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.xt.extrinsic_count().await
	}

	/// Counts events emitted by this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of events emitted in the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn event_count(&self) -> Result<u32, Error> {
		self.xt.event_count().await
	}

	/// Retrieves the block weight grouped by dispatch class.
	///
	/// # Returns
	/// - `Ok(PerDispatchClassWeight)`: Weight totals reported by the runtime.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		self.xt.weight().await
	}

	/// Aggregates the weight consumed by extrinsics in this block.
	///
	/// # Returns
	/// - `Ok(Weight)`: Total weight indicated by success and failure events.
	/// - `Err(Error)`: Event retrieval or decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests via the decoded extrinsics helper and may retry as configured.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		self.xt.extrinsic_weight().await
	}
}

/// View that fetches events emitted by a block, optionally filtered by extrinsic.
pub struct Events {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl Events {
	/// Creates an event view for the given block.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch event data.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Helper for retrieving events scoped to the block.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Events { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Returns events emitted by a specific extrinsic index.
	///
	/// # Parameters
	/// - `tx_index`: Index of the extrinsic whose events should be returned.
	///
	/// # Returns
	/// - `Ok(AllEvents)`: Wrapper around events emitted by the extrinsic (may be empty).
	/// - `Err(Error)`: RPC retrieval or event decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn extrinsic(&self, tx_index: u32) -> Result<AllEvents, Error> {
		let events = self.all(tx_index.into()).await?;
		Ok(AllEvents::new(events))
	}

	/// Returns system-level events that are not tied to extrinsics.
	///
	/// # Returns
	/// - `Ok(AllEvents)`: Wrapper around system events (may be empty).
	/// - `Err(Error)`: RPC retrieval or event decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn system(&self) -> Result<AllEvents, Error> {
		let events = self.all(rpc::EventFilter::OnlyNonExtrinsics).await?;
		let events: Vec<Event> = events
			.into_iter()
			.filter(|x| x.phase.extrinsic_index().is_none())
			.collect();

		Ok(AllEvents::new(events))
	}

	/// Fetches all events for the block using the given filter.
	///
	/// # Parameters
	/// - `filter`: Filter describing which phases or extrinsics to include.
	///
	/// # Returns
	/// - `Ok(Vec<Event>)`: Zero or more events matching the filter.
	/// - `Err(Error)`: RPC retrieval or event decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn all(&self, filter: rpc::EventFilter) -> Result<Vec<Event>, Error> {
		let opts = rpc::EventOpts {
			filter: Some(filter),
			enable_encoding: Some(true),
			enable_decoding: Some(false),
		};

		let block_phase_events = self
			.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.system_fetch_events(self.block_id.clone(), opts)
			.await?;

		let mut result: Vec<Event> = Vec::new();
		for block_phase_event in block_phase_events {
			let phase = block_phase_event.phase;

			for mut phase_event in block_phase_event.events {
				let Some(data) = phase_event.encoded_data.take() else {
					return Err(
						RpcError::ExpectedData("The node did not return encoded data for this event.".into()).into()
					);
				};

				let all_event = Event {
					index: phase_event.index,
					pallet_id: phase_event.pallet_id,
					variant_id: phase_event.variant_id,
					data,
					phase,
				};
				result.push(all_event);
			}
		}

		Ok(result)
	}

	/// Fetches raw event data with full RPC control.
	///
	/// # Parameters
	/// - `opts`: RPC options specifying filters and encoding preferences.
	///
	/// # Returns
	/// - `Ok(Vec<BlockPhaseEvent>)`: Raw events grouped by block phase.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn raw(&self, opts: rpc::EventOpts) -> Result<Vec<BlockPhaseEvent>, Error> {
		let result = self
			.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.system_fetch_events(self.block_id.clone(), opts)
			.await?;

		Ok(result)
	}

	/// Overrides retry behaviour for event lookups.
	///
	/// # Parameters
	/// - `value`: Retry override (`Some(true)` to force retries, `Some(false)` to disable, `None` to inherit).
	///
	/// # Returns
	/// - `()`: The new retry preference is stored.
	///
	/// # Side Effects
	/// - Updates internal state so future RPC requests honour the override.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Reports whether event queries retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

	/// Aggregates weight consumed by extrinsics using emitted events.
	///
	/// # Returns
	/// - `Ok(Weight)`: Summed weights derived from `ExtrinsicSuccess` and `ExtrinsicFailed` events.
	/// - `Err(Error)`: Event retrieval or decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		use avail::system::events::{ExtrinsicFailed, ExtrinsicSuccess};

		let mut weight = Weight::default();
		let events = self.all(rpc::EventFilter::OnlyExtrinsics).await?;
		for event in events {
			if event.phase.extrinsic_index().is_none() {
				continue;
			}

			let header = (event.pallet_id, event.variant_id);
			if header == ExtrinsicSuccess::HEADER_INDEX {
				let e = ExtrinsicSuccess::from_event(event.data).map_err(Error::Other)?;
				weight.ref_time += e.dispatch_info.weight.ref_time;
				weight.proof_size += e.dispatch_info.weight.proof_size;
			} else if header == ExtrinsicFailed::HEADER_INDEX {
				let e = ExtrinsicFailed::from_event(event.data).map_err(Error::Other)?;
				weight.ref_time += e.dispatch_info.weight.ref_time;
				weight.proof_size += e.dispatch_info.weight.proof_size;
			}
		}

		Ok(weight)
	}

	/// Counts events emitted by this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of events emitted in the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues an RPC request and may retry as configured.
	pub async fn event_count(&self) -> Result<u32, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_event_count(self.block_id.clone()).await
	}
}

#[derive(Debug, Default, Clone)]
pub struct ExtrinsicsOpts {
	pub filter: Option<ExtrinsicFilter>,
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
}

impl ExtrinsicsOpts {
	/// Creates a builder with all filters unset.
	///
	/// # Returns
	/// - `Self`: Options builder with default values.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the expected nonce filter.
	///
	/// # Parameters
	/// - `value`: Nonce that matching extrinsics must carry.
	///
	/// # Returns
	/// - `Self`: Builder with the nonce filter applied.
	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	/// Sets the application identifier filter.
	///
	/// # Parameters
	/// - `value`: Application identifier required for matching extrinsics.
	///
	/// # Returns
	/// - `Self`: Builder with the application filter applied.
	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	/// Sets the signer address filter.
	///
	/// # Parameters
	/// - `value`: Address (SS58 format) required for matching extrinsics.
	///
	/// # Returns
	/// - `Self`: Builder with the address filter applied.
	pub fn ss58_address(mut self, value: impl Into<String>) -> Self {
		self.ss58_address = Some(value.into());
		self
	}

	/// Sets the primary transaction filter.
	///
	/// # Parameters
	/// - `value`: Filter describing the target extrinsics (hash, index, or number).
	///
	/// # Returns
	/// - `Self`: Builder with the transaction filter applied.
	pub fn filter(mut self, value: impl Into<ExtrinsicFilter>) -> Self {
		self.filter = Some(value.into());
		self
	}

	/// Converts the builder into RPC options with the requested encoding.
	///
	/// # Parameters
	/// - `encode_as`: Encoding preference for the RPC response.
	///
	/// # Returns
	/// - `rpc::ExtrinsicOpts`: Ready-to-send RPC configuration.
	pub fn to_rpc_opts(self, encode_as: EncodeSelector) -> rpc::ExtrinsicOpts {
		rpc::ExtrinsicOpts {
			transaction_filter: self.filter.unwrap_or_default(),
			ss58_address: self.ss58_address,
			app_id: self.app_id,
			nonce: self.nonce,
			encode_as,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Metadata {
	/// Hash of the extrinsic.
	pub ext_hash: H256,
	/// Index of the extrinsic within the block.
	pub ext_index: u32,
	/// Pallet identifier associated with the call.
	pub pallet_id: u8,
	/// Variant within the pallet identifying the call.
	pub variant_id: u8,
	/// Block identifier (hash or number) where the extrinsic resides.
	pub block_id: HashNumber,
}

impl Metadata {
	/// Wraps metadata about an extrinsic inside a block.
	///
	/// # Parameters
	/// - `ext_hash`: Hash of the extrinsic.
	/// - `ext_index`: Index of the extrinsic within the block.
	/// - `pallet_id`: Pallet identifier associated with the call.
	/// - `variant_id`: Variant identifier within the pallet.
	/// - `block_id`: Hash or number of the block containing the extrinsic.
	///
	/// # Returns
	/// - `Self`: Metadata wrapper encapsulating the supplied values.
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, block_id: HashNumber) -> Self {
		Self { ext_hash, ext_index, pallet_id, variant_id, block_id }
	}
}

#[derive(Debug, Clone)]
pub struct EncodedExtrinsic {
	/// SCALE-encoded string representation of the extrinsic.
	pub data: String,
	/// Associated metadata describing the extrinsic.
	pub metadata: Metadata,
	/// Optional signer payload supplied by the node.
	pub signer_payload: Option<SignerPayload>,
}

impl EncodedExtrinsic {
	/// Creates a raw extrinsic wrapper.
	///
	/// # Parameters
	/// - `data`: SCALE-encoded extrinsic payload as a string.
	/// - `metadata`: Metadata describing the extrinsic.
	/// - `signer_payload`: Optional signer metadata supplied by the node.
	///
	/// # Returns
	/// - `Self`: Encoded extrinsic wrapper containing the provided data.
	pub fn new(data: String, metadata: Metadata, signer_payload: Option<SignerPayload>) -> Self {
		Self { data, metadata, signer_payload }
	}

	/// Fetches events emitted by this extrinsic.
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
		let events = Events::new(client, self.metadata.block_id)
			.extrinsic(self.ext_index())
			.await?;

		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	/// Returns the index of this extrinsic inside the block.
	///
	/// # Returns
	/// - `u32`: Index of the extrinsic within the block.
	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	/// Returns the extrinsic hash.
	///
	/// # Returns
	/// - `H256`: Hash of the extrinsic.
	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	/// Returns the application id if the signer payload provided it.
	///
	/// # Returns
	/// - `Some(u32)`: Application identifier from the signer payload.
	/// - `None`: Signer payload was absent.
	pub fn app_id(&self) -> Option<u32> {
		Some(self.signer_payload.as_ref()?.app_id)
	}

	/// Returns the nonce if the signer payload provided it.
	///
	/// # Returns
	/// - `Some(u32)`: Nonce from the signer payload.
	/// - `None`: Signer payload was absent.
	pub fn nonce(&self) -> Option<u32> {
		Some(self.signer_payload.as_ref()?.nonce)
	}

	/// Returns the ss58 address if the signer payload provided it.
	///
	/// # Returns
	/// - `Some(String)`: SS58 address supplied by the signer payload.
	/// - `None`: Signer payload was absent.
	pub fn ss58_address(&self) -> Option<String> {
		self.signer_payload.as_ref()?.ss58_address.clone()
	}

	/// Converts the encoded extrinsic into a signed variant when possible.
	///
	/// # Returns
	/// - `Ok(SignedExtrinsic<T>)`: Signed extrinsic decoded from the encoded payload.
	/// - `Err(String)`: The extrinsic was unsigned or failed to decode as `T`.
	pub fn as_signed<T: HasHeader + Decode>(&self) -> Result<SignedExtrinsic<T>, String> {
		SignedExtrinsic::<T>::try_from(self)
	}

	/// Converts the encoded extrinsic into a decoded extrinsic wrapper.
	///
	/// # Returns
	/// - `Ok(Extrinsic<T>)`: Decoded extrinsic containing the call and metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	pub fn as_extrinsic<T: HasHeader + Decode>(&self) -> Result<Extrinsic<T>, String> {
		Extrinsic::<T>::try_from(self)
	}

	/// Checks whether the encoded extrinsic matches the header index for `T`.
	///
	/// # Returns
	/// - `true`: The extrinsic matches `T::HEADER_INDEX`.
	/// - `false`: The header indices differ.
	pub fn is<T: HasHeader>(&self) -> bool {
		self.metadata.pallet_id == T::HEADER_INDEX.0 && self.metadata.variant_id == T::HEADER_INDEX.1
	}

	/// Returns the pallet and variant identifiers stored in the metadata.
	///
	/// # Returns
	/// - `(u8, u8)`: Tuple containing `(pallet_id, variant_id)`.
	pub fn header(&self) -> (u8, u8) {
		(self.metadata.pallet_id, self.metadata.variant_id)
	}
}

/// Decoded extrinsic along with metadata and optional signature.
#[derive(Debug, Clone)]
pub struct Extrinsic<T: HasHeader + Decode> {
	/// Optional signature associated with the extrinsic.
	pub signature: Option<ExtrinsicSignature>,
	/// Decoded runtime call payload.
	pub call: T,
	/// Metadata describing where the extrinsic was found.
	pub metadata: Metadata,
}

impl<T: HasHeader + Decode> Extrinsic<T> {
	/// Creates an extrinsic wrapper from decoded data.
	///
	/// # Parameters
	/// - `signature`: Optional signature attached to the extrinsic.
	/// - `call`: Decoded call payload.
	/// - `metadata`: Metadata describing the extrinsic context.
	///
	/// # Returns
	/// - `Self`: Decoded extrinsic wrapper containing the provided data.
	pub fn new(signature: Option<ExtrinsicSignature>, call: T, metadata: Metadata) -> Self {
		Self { signature, call, metadata }
	}

	/// Fetches events emitted by this extrinsic.
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
		let events = Events::new(client, self.metadata.block_id)
			.extrinsic(self.ext_index())
			.await?;

		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for extrinsic".into()).into());
		};

		Ok(events)
	}

	/// Returns the index of this extrinsic inside the block.
	///
	/// # Returns
	/// - `u32`: Index of the extrinsic within the block.
	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	/// Returns the extrinsic hash.
	///
	/// # Returns
	/// - `H256`: Hash of the extrinsic.
	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	/// Returns the application id if the extrinsic was signed.
	///
	/// # Returns
	/// - `Some(u32)`: Application identifier from the signature.
	/// - `None`: Extrinsic was unsigned.
	pub fn app_id(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.tx_extra.app_id)
	}

	/// Returns the nonce if the extrinsic was signed.
	///
	/// # Returns
	/// - `Some(u32)`: Nonce from the signature.
	/// - `None`: Extrinsic was unsigned.
	pub fn nonce(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.tx_extra.nonce)
	}

	/// Returns the tip if the extrinsic was signed.
	///
	/// # Returns
	/// - `Some(u128)`: Tip reported by the signature.
	/// - `None`: Extrinsic was unsigned.
	pub fn tip(&self) -> Option<u128> {
		Some(self.signature.as_ref()?.tx_extra.tip)
	}

	/// Returns the signer as an ss58 string when available.
	///
	/// # Returns
	/// - `Some(String)`: SS58-encoded signer address.
	/// - `None`: Extrinsic was unsigned or used a non-`Id` multi-address.
	pub fn ss58_address(&self) -> Option<String> {
		match &self.signature.as_ref()?.address {
			MultiAddress::Id(x) => Some(std::format!("{}", x)),
			_ => None,
		}
	}

	/// Converts the extrinsic into a signed variant when a signature is present.
	///
	/// # Returns
	/// - `Ok(SignedExtrinsic<T>)`: Signed wrapper containing the same call and metadata.
	/// - `Err(String)`: Extrinsic was unsigned, so a signed variant cannot be produced.
	pub fn as_signed(&self) -> Result<SignedExtrinsic<T>, String>
	where
		T: Clone,
	{
		SignedExtrinsic::<T>::try_from(self)
	}
}

impl<T: HasHeader + Decode> TryFrom<EncodedExtrinsic> for Extrinsic<T> {
	type Error = String;

	/// Decodes an encoded extrinsic into an `Extrinsic<T>`.
	///
	/// # Parameters
	/// - `value`: Encoded extrinsic containing the SCALE payload.
	///
	/// # Returns
	/// - `Ok(Self)`: Decoded extrinsic with optional signature and metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	fn try_from(value: EncodedExtrinsic) -> Result<Self, Self::Error> {
		let extrinsic = CoreExtrinsic::<T>::try_from(value.data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata))
	}
}

impl<T: HasHeader + Decode> TryFrom<&EncodedExtrinsic> for Extrinsic<T> {
	type Error = String;

	/// Decodes a borrowed encoded extrinsic into an `Extrinsic<T>`.
	///
	/// # Parameters
	/// - `value`: Borrowed encoded extrinsic containing the SCALE payload.
	///
	/// # Returns
	/// - `Ok(Self)`: Decoded extrinsic with optional signature and cloned metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	fn try_from(value: &EncodedExtrinsic) -> Result<Self, Self::Error> {
		let extrinsic = CoreExtrinsic::<T>::try_from(value.data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata.clone()))
	}
}

/// Block Transaction is the same as Block Signed Extrinsic
#[derive(Debug, Clone)]
pub struct SignedExtrinsic<T: HasHeader + Decode> {
	/// Signature proving authorship of the extrinsic.
	pub signature: ExtrinsicSignature,
	/// Decoded runtime call payload.
	pub call: T,
	/// Metadata describing where the extrinsic was found.
	pub metadata: Metadata,
}

impl<T: HasHeader + Decode> SignedExtrinsic<T> {
	/// Creates a transaction wrapper from decoded data.
	///
	/// # Parameters
	/// - `signature`: Signature associated with the extrinsic.
	/// - `call`: Decoded call payload.
	/// - `metadata`: Metadata describing the extrinsic context.
	///
	/// # Returns
	/// - `Self`: Signed extrinsic wrapper containing the provided data.
	pub fn new(signature: ExtrinsicSignature, call: T, metadata: Metadata) -> Self {
		Self { signature, call, metadata }
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
		let events = Events::new(client, self.metadata.block_id)
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

	/// Returns the application id for this transaction.
	///
	/// # Returns
	/// - `u32`: Application identifier recorded in the signature.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn app_id(&self) -> u32 {
		self.signature.tx_extra.app_id
	}

	/// Returns the signer nonce for this transaction.
	///
	/// # Returns
	/// - `u32`: Nonce recorded in the signature.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn nonce(&self) -> u32 {
		self.signature.tx_extra.nonce
	}

	/// Returns the paid tip for this transaction.
	///
	/// # Returns
	/// - `u128`: Tip recorded in the signature.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn tip(&self) -> u128 {
		self.signature.tx_extra.tip
	}

	/// Returns the signer as an ss58 string when available.
	///
	/// # Returns
	/// - `Some(String)`: SS58-encoded signer address.
	/// - `None`: Signer address is not stored as an `Id`.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn ss58_address(&self) -> Option<String> {
		match &self.signature.address {
			MultiAddress::Id(x) => Some(std::format!("{}", x)),
			_ => None,
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<Extrinsic<T>> for SignedExtrinsic<T> {
	type Error = String;

	/// Converts a decoded extrinsic into a signed extrinsic when a signature is present.
	///
	/// # Parameters
	/// - `value`: Decoded extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic carrying the call and metadata.
	/// - `Err(String)`: The extrinsic was unsigned.
	fn try_from(value: Extrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature, value.call, value.metadata))
	}
}

impl<T: HasHeader + Decode + Clone> TryFrom<&Extrinsic<T>> for SignedExtrinsic<T> {
	type Error = String;

	/// Converts a borrowed extrinsic into a signed extrinsic when a signature is present.
	///
	/// # Parameters
	/// - `value`: Borrowed extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic with cloned call and metadata.
	/// - `Err(String)`: The extrinsic was unsigned.
	fn try_from(value: &Extrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = &value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature.clone(), value.call.clone(), value.metadata.clone()))
	}
}

impl<T: HasHeader + Decode> TryFrom<EncodedExtrinsic> for SignedExtrinsic<T> {
	type Error = String;

	/// Decodes an encoded extrinsic into a signed extrinsic.
	///
	/// # Parameters
	/// - `value`: Encoded extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic decoded from the payload.
	/// - `Err(String)`: Decoding failed or the extrinsic was unsigned.
	fn try_from(value: EncodedExtrinsic) -> Result<Self, Self::Error> {
		let ext = Extrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<&EncodedExtrinsic> for SignedExtrinsic<T> {
	type Error = String;

	/// Decodes a borrowed encoded extrinsic into a signed extrinsic.
	///
	/// # Parameters
	/// - `value`: Borrowed encoded extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic decoded from the payload.
	/// - `Err(String)`: Decoding failed or the extrinsic was unsigned.
	fn try_from(value: &EncodedExtrinsic) -> Result<Self, Self::Error> {
		let ext = Extrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

#[derive(Debug, Clone)]
pub struct Event {
	/// Phase of block execution in which the event occurred.
	pub phase: RuntimePhase,
	/// Sequential index of the event within the phase.
	pub index: u32,
	/// Identifier of the emitting pallet.
	pub pallet_id: u8,
	/// Identifier of the variant inside the pallet.
	pub variant_id: u8,
	/// SCALE-encoded payload containing event data.
	pub data: String,
}

#[derive(Debug, Clone)]
pub struct AllEvents {
	/// Collection of decoded events preserved in original order.
	pub events: Vec<Event>,
}

impl AllEvents {
	/// Wraps decoded events.
	///
	/// # Parameters
	/// - `events`: Collection of decoded events to wrap.
	///
	/// # Returns
	/// - `Self`: Wrapper exposing helper methods for event queries.
	pub fn new(events: Vec<Event>) -> Self {
		Self { events }
	}

	/// Returns the first event matching the requested type.
	///
	/// # Returns
	/// - `Some(T)`: First event decoded as the requested type.
	/// - `None`: No matching event was found or decoding failed.
	pub fn first<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns the last event matching the requested type.
	///
	/// # Returns
	/// - `Some(T)`: Last event decoded as the requested type.
	/// - `None`: No matching event was found or decoding failed.
	pub fn last<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.rev()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns every event matching the requested type.
	///
	/// # Returns
	/// - `Ok(Vec<T>)`: Zero or more events decoded as the requested type.
	/// - `Err(Error)`: Event decoding failed.
	pub fn all<T: HasHeader + codec::Decode>(&self) -> Result<Vec<T>, Error> {
		let mut result = Vec::new();
		for event in &self.events {
			if event.pallet_id != T::HEADER_INDEX.0 || event.variant_id != T::HEADER_INDEX.1 {
				continue;
			}

			let decoded = T::from_event(event.data.as_str()).map_err(|x| Error::User(UserError::Decoding(x)))?;
			result.push(decoded);
		}

		Ok(result)
	}

	/// Checks if an `ExtrinsicSuccess` event exists.
	///
	/// # Returns
	/// - `true`: At least one `ExtrinsicSuccess` event is present.
	/// - `false`: No such events were recorded.
	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	/// Checks if an `ExtrinsicFailed` event exists.
	///
	/// # Returns
	/// - `true`: At least one `ExtrinsicFailed` event is present.
	/// - `false`: No such events were recorded.
	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	/// Returns whether a proxy call succeeded, when present.
	///
	/// # Returns
	/// - `Some(true)`: A proxy call executed successfully.
	/// - `Some(false)`: A proxy call executed but failed.
	/// - `None`: No proxy execution event was recorded.
	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::proxy::events::ProxyExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns whether a multisig call succeeded, when present.
	///
	/// # Returns
	/// - `Some(true)`: A multisig call executed successfully.
	/// - `Some(false)`: A multisig call executed but failed.
	/// - `None`: No multisig execution event was recorded.
	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::multisig::events::MultisigExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns true when at least one event of the given type exists.
	///
	/// # Returns
	/// - `true`: At least one matching event exists.
	/// - `false`: No matching events were recorded.
	pub fn is_present<T: HasHeader>(&self) -> bool {
		self.count::<T>() > 0
	}

	/// Returns true when the given pallet and variant combination appears.
	///
	/// # Parameters
	/// - `pallet_id`: Target pallet identifier.
	/// - `variant_id`: Target variant identifier.
	///
	/// # Returns
	/// - `true`: At least one matching event exists.
	/// - `false`: No matching events were recorded.
	pub fn is_present_parts(&self, pallet_id: u8, variant_id: u8) -> bool {
		self.count_parts(pallet_id, variant_id) > 0
	}

	/// Counts how many times the given event type appears.
	///
	/// # Returns
	/// - `u32`: Number of matching events recorded.
	pub fn count<T: HasHeader>(&self) -> u32 {
		self.count_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1)
	}

	/// Counts how many events match the pallet and variant combo.
	///
	/// # Parameters
	/// - `pallet_id`: Target pallet identifier.
	/// - `variant_id`: Target variant identifier.
	///
	/// # Returns
	/// - `u32`: Number of matching events recorded.
	pub fn count_parts(&self, pallet_id: u8, variant_id: u8) -> u32 {
		let mut count = 0;
		self.events.iter().for_each(|x| {
			if x.pallet_id == pallet_id && x.variant_id == variant_id {
				count += 1
			}
		});

		count
	}

	/// Returns the number of cached events.
	///
	/// # Returns
	/// - `usize`: Total events stored in the wrapper.
	pub fn len(&self) -> usize {
		self.events.len()
	}

	/// Reports whether any events are cached.
	///
	/// # Returns
	/// - `true`: The wrapper contains no events.
	/// - `false`: At least one event is stored.
	pub fn is_empty(&self) -> bool {
		self.events.is_empty()
	}
}

mod shared {
	pub use super::*;

	/// Fetches the block header for the provided identifier.
	///
	/// # Parameters
	/// - `client`: RPC client used to perform the header query.
	/// - `block_id`: Hash or number identifying the target block.
	///
	/// # Returns
	/// - `Ok(AvailHeader)`: Header returned by the node.
	/// - `Err(Error)`: The RPC call failed or the block could not be resolved.
	///
	/// # Side Effects
	/// - Performs an RPC call through the client's chain interface.
	pub async fn header(client: &Client, block_id: HashStringNumber) -> Result<AvailHeader, Error> {
		let header = client.chain().block_header(Some(block_id.clone())).await?;
		let Some(header) = header else {
			return Err(Error::User(UserError::Other(std::format!(
				"No block header found for block id: {}",
				block_id
			))));
		};

		Ok(header)
	}
}
