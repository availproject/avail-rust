//! Convenience helpers for inspecting block data, extrinsics, and events via RPC.

use crate::{Client, Error, UserError};
use avail_rust_core::{
	EncodeSelector, Extrinsic, ExtrinsicSignature, H256, HasHeader, HashNumber, MultiAddress, RpcError,
	TransactionEventDecodable, avail,
	grandpa::GrandpaJustification,
	rpc::{self, ExtrinsicFilter, SignerPayload},
	types::HashStringNumber,
};
use codec::Decode;

/// High-level handle bound to a specific block id (height or hash).
pub struct BlockApi {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl BlockApi {
	/// Creates a block helper for the given height or hash.
	///
	/// No network calls are issued up front; the `block_id` is stored for later RPC queries. Use the
	/// view helpers such as [`BlockApi::tx`] or [`BlockApi::events`] to fetch concrete data.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		BlockApi { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Returns a view that focuses on decoded transactions.
	///
	/// The returned [`BlockWithTx`] shares this helper's retry configuration.
	pub fn tx(&self) -> BlockWithTx {
		BlockWithTx::new(self.client.clone(), self.block_id.clone())
	}

	/// Returns a view that focuses on decoded extrinsics while retaining signature metadata.
	pub fn ext(&self) -> BlockWithExt {
		BlockWithExt::new(self.client.clone(), self.block_id.clone())
	}

	/// Returns a view that keeps extrinsics as raw bytes and optional signer payload information.
	pub fn raw_ext(&self) -> BlockWithRawExt {
		BlockWithRawExt::new(self.client.clone(), self.block_id.clone())
	}

	/// Returns a helper for fetching events from this block.
	pub fn events(&self) -> BlockEvents {
		BlockEvents::new(self.client.clone(), self.block_id.clone())
	}

	/// Controls retry behaviour for follow-up RPC calls made through this block helper.
	///
	/// - `Some(true)` forces retries regardless of the client's global setting.
	/// - `Some(false)` disables retries entirely.
	/// - `None` keeps the client's default configuration.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Fetches the GRANDPA justification for this block when available.
	///
	/// # Returns
	/// - `Ok(Some(GrandpaJustification))` when the runtime provides a justification.
	/// - `Ok(None)` when no justification exists for the requested block.
	/// - `Err(Error)` if the RPC layer fails or the supplied block id cannot be resolved.
	pub async fn justification(&self) -> Result<Option<GrandpaJustification>, Error> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let at = match block_id {
			HashNumber::Hash(h) => self
				.client
				.chain()
				.retry_on(retry_on_error, None)
				.block_height(h)
				.await?
				.ok_or(Error::Other("Failed to find block from the provided hash".into()))?,
			HashNumber::Number(n) => n,
		};

		self.client
			.chain()
			.retry_on(retry_on_error, None)
			.grandpa_block_justification(at)
			.await
			.map_err(|e| e.into())
	}
}

/// View of block extrinsics as raw payloads with associated metadata.
pub struct BlockWithRawExt {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl BlockWithRawExt {
	/// Builds a raw extrinsic view for the given block.
	///
	/// The `block_id` may be a height or hash; conversions happen lazily when RPCs are executed.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Finds a specific extrinsic and returns it in the requested format.
	///
	/// # Returns
	/// - `Ok(Some(BlockRawExtrinsic))` when the extrinsic is found.
	/// - `Ok(None)` when no extrinsic matches the provided identifier.
	/// - `Err(Error)` when decoding the identifier fails or the RPC call errors.
	pub async fn get(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
		encode_as: EncodeSelector,
	) -> Result<Option<BlockRawExtrinsic>, Error> {
		async fn inner(
			s: &BlockWithRawExt,
			extrinsic_id: HashStringNumber,
			encode_as: EncodeSelector,
		) -> Result<Option<BlockRawExtrinsic>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(|x| UserError::Decoding(x))?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			let mut opts = BlockExtOptionsExpanded::default();
			opts.filter = Some(filter);
			opts.encode_as = Some(encode_as);
			Ok(s.first(opts).await?)
		}

		inner(&self, extrinsic_id.into(), encode_as).await
	}

	/// Returns the first matching extrinsic, if any.
	///
	/// # Returns
	/// - `Ok(Some(BlockRawExtrinsic))` with metadata and optional payload.
	/// - `Ok(None)` when nothing matches the provided filters.
	/// - `Err(Error)` on RPC failures or when the block identifier cannot be decoded.
	pub async fn first(&self, mut opts: BlockExtOptionsExpanded) -> Result<Option<BlockRawExtrinsic>, Error> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		if opts.encode_as.is_none() {
			opts.encode_as = Some(EncodeSelector::Extrinsic)
		}

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let mut result = self
			.client
			.chain()
			.retry_on(retry_on_error, None)
			.system_fetch_extrinsics(block_id, opts.into())
			.await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let metadata =
			BlockExtrinsicMetadata::new(first.ext_hash, first.ext_index, first.pallet_id, first.variant_id, block_id);
		let ext = BlockRawExtrinsic::new(first.data.take(), metadata, first.signer_payload.take());

		Ok(Some(ext))
	}

	/// Returns the last matching extrinsic, if any.
	///
	/// Return semantics mirror [`BlockWithRawExt::first`], but the final matching element is returned.
	pub async fn last(&self, mut opts: BlockExtOptionsExpanded) -> Result<Option<BlockRawExtrinsic>, Error> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		if opts.encode_as.is_none() {
			opts.encode_as = Some(EncodeSelector::Extrinsic)
		}

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let mut result = self
			.client
			.chain()
			.retry_on(retry_on_error, None)
			.system_fetch_extrinsics(block_id, opts.into())
			.await?;

		let Some(last) = result.last_mut() else {
			return Ok(None);
		};

		let metadata =
			BlockExtrinsicMetadata::new(last.ext_hash, last.ext_index, last.pallet_id, last.variant_id, block_id);
		let ext = BlockRawExtrinsic::new(last.data.take(), metadata, last.signer_payload.take());

		Ok(Some(ext))
	}

	/// Returns all matching extrinsics.
	///
	/// The resulting vector may be empty when no extrinsics satisfy the filters.
	pub async fn all(&self, mut opts: BlockExtOptionsExpanded) -> Result<Vec<BlockRawExtrinsic>, Error> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		if opts.encode_as.is_none() {
			opts.encode_as = Some(EncodeSelector::Extrinsic)
		}

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let result = self
			.client
			.chain()
			.retry_on(retry_on_error, None)
			.system_fetch_extrinsics(block_id, opts.into())
			.await?;

		let result = result
			.into_iter()
			.map(|x| {
				let metadata =
					BlockExtrinsicMetadata::new(x.ext_hash, x.ext_index, x.pallet_id, x.variant_id, block_id);
				BlockRawExtrinsic::new(x.data, metadata, x.signer_payload)
			})
			.collect();

		Ok(result)
	}

	/// Counts matching extrinsics without fetching the payloads.
	///
	/// Equivalent to `self.all(opts).await.map(|v| v.len())` but avoids transferring payload bytes.
	pub async fn count(&self, mut opts: BlockExtOptionsExpanded) -> Result<usize, Error> {
		opts.encode_as = Some(EncodeSelector::None);

		let result = self.all(opts).await?;
		Ok(result.len())
	}

	/// Checks whether at least one extrinsic matches.
	pub async fn exists(&self, mut opts: BlockExtOptionsExpanded) -> Result<bool, Error> {
		opts.encode_as = Some(EncodeSelector::None);

		let result = self.first(opts).await?;
		Ok(result.is_some())
	}

	/// Controls retry behaviour for fetching raw extrinsics: `Some(true)` forces retries,
	/// `Some(false)` disables them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}
}

/// View of block extrinsics decoded into calls and optional signatures.
pub struct BlockWithExt {
	rxt: BlockWithRawExt,
}

impl BlockWithExt {
	/// Builds a decoded extrinsic view for the given block.
	///
	/// Decoding happens lazily as individual queries are made.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { rxt: BlockWithRawExt::new(client, block_id) }
	}

	/// Fetches a specific extrinsic by id.
	///
	/// # Returns
	/// - `Ok(Some(BlockExtrinsic<T>))` when the extrinsic exists and decodes as `T`.
	/// - `Ok(None)` when no extrinsic matches the identifier or filters.
	/// - `Err(Error)` if the RPC call fails or decoding the identifier/payload fails.
	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<BlockExtrinsic<T>>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &BlockWithExt,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<BlockExtrinsic<T>>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			let filter = Some(filter);
			Ok(s.first::<T>(BlockExtOptionsSimple { filter, ..Default::default() })
				.await?)
		}

		inner::<T>(&self, extrinsic_id.into()).await
	}

	/// Returns the first matching extrinsic decoded into the target type.
	///
	/// # Returns
	/// - `Ok(Some(BlockExtrinsic<T>))` when an extrinsic matches the filters.
	/// - `Ok(None)` when nothing matches.
	/// - `Err(Error)` if RPC retrieval fails or decoding the extrinsic as `T` fails.
	pub async fn first<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Option<BlockExtrinsic<T>>, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let first = self.rxt.first(opts).await?;
		let Some(first) = first else {
			return Ok(None);
		};

		let Some(data) = first.data else {
			return Err(RpcError::ExpectedData("Fetched raw extrinsic had no data.".into()).into());
		};

		let ext = Extrinsic::<T>::try_from(data.as_str()).map_err(UserError::Decoding)?;
		let ext = BlockExtrinsic::new(ext.signature, ext.call, first.metadata);

		Ok(Some(ext))
	}

	/// Returns the last matching extrinsic decoded into the target type.
	///
	/// Return semantics mirror [`BlockWithExt::first`], but the final matching extrinsic is returned.
	pub async fn last<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Option<BlockExtrinsic<T>>, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let last = self.rxt.last(opts).await?;
		let Some(last) = last else {
			return Ok(None);
		};

		let Some(data) = last.data else {
			return Err(RpcError::ExpectedData("Fetched raw extrinsic had no data.".into()).into());
		};

		let ext = Extrinsic::<T>::try_from(data.as_str()).map_err(UserError::Decoding)?;
		let ext = BlockExtrinsic::new(ext.signature, ext.call, last.metadata);
		Ok(Some(ext))
	}

	/// Returns every matching extrinsic decoded into the target type.
	///
	/// The result may be empty if no extrinsics match. Decoding failures are surfaced as `Err(Error)`.
	pub async fn all<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Vec<BlockExtrinsic<T>>, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let all = self.rxt.all(opts).await?;
		let mut result = Vec::with_capacity(all.len());
		for raw_ext in all {
			let Some(data) = raw_ext.data else {
				return Err(RpcError::ExpectedData("Fetched raw extrinsic had no data.".into()).into());
			};
			let ext = Extrinsic::<T>::try_from(data.as_str()).map_err(UserError::Decoding)?;
			let ext = BlockExtrinsic::new(ext.signature, ext.call, raw_ext.metadata);
			result.push(ext);
		}

		Ok(result)
	}

	/// Counts matching extrinsics without decoding the payloads.
	///
	/// This still performs an RPC round-trip but avoids transferring the encoded call data.
	pub async fn count<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<usize, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		opts.encode_as = Some(EncodeSelector::None);
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		return self.rxt.count(opts).await;
	}

	/// Checks whether any extrinsic matches the filters.
	///
	/// Equivalent to calling [`BlockWithExt::first`] and testing the result for `Some`.
	pub async fn exists<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<bool, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		opts.encode_as = Some(EncodeSelector::None);
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		return self.rxt.exists(opts).await;
	}

	/// Controls retry behaviour for decoded-extrinsic lookups: `Some(true)` forces retries,
	/// `Some(false)` disables them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.rxt.set_retry_on_error(value);
	}
}

/// View of block extrinsics restricted to signed transactions.
pub struct BlockWithTx {
	ext: BlockWithExt,
}

impl BlockWithTx {
	/// Builds a signed transaction view for the given block.
	///
	/// Only signed extrinsics will be surfaced; unsigned extrinsics produce an `Error` when decoding.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { ext: BlockWithExt::new(client, block_id) }
	}

	/// Fetches a signed transaction by id.
	///
	/// # Returns
	/// - `Ok(Some(BlockTransaction<T>))` when the extrinsic exists and carries a signature.
	/// - `Ok(None)` when no extrinsic matches the identifier.
	/// - `Err(Error)` when the extrinsic exists but is unsigned or cannot be decoded as `T`.
	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<BlockTransaction<T>>, Error> {
		let ext = self.ext.get(extrinsic_id).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into());
		};

		let ext = BlockTransaction::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	/// Returns the first matching signed transaction.
	///
	/// Unsigned extrinsics encountered during decoding produce an `Error`.
	pub async fn first<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Option<BlockTransaction<T>>, Error> {
		let ext = self.ext.first(opts).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into());
		};

		let ext = BlockTransaction::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	/// Returns the last matching signed transaction.
	///
	/// Return semantics mirror [`BlockWithTx::first`], but returns the final matching transaction.
	pub async fn last<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Option<BlockTransaction<T>>, Error> {
		let ext = self.ext.last(opts).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into());
		};

		let ext = BlockTransaction::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	/// Returns every matching signed transaction.
	///
	/// Decoding stops early with an `Error` if any extrinsic lacks a signature or fails to decode as `T`.
	pub async fn all<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Vec<BlockTransaction<T>>, Error> {
		let all = self.ext.all::<T>(opts).await?;
		let mut result = Vec::with_capacity(all.len());
		for ext in all {
			let Some(signature) = ext.signature else {
				return Err(UserError::Other("Extrinsic is unsigned; cannot decode it as a signed transaction.".into()).into());
			};
			result.push(BlockTransaction::new(signature, ext.call, ext.metadata));
		}

		Ok(result)
	}

	/// Counts matching signed transactions.
	pub async fn count<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<usize, Error> {
		self.ext.count::<T>(opts).await
	}

	/// Checks whether any signed transaction matches the filters.
	pub async fn exists<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<bool, Error> {
		self.ext.exists::<T>(opts).await
	}

	/// Controls retry behaviour for signed-transaction lookups: `Some(true)` forces retries,
	/// `Some(false)` disables them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.ext.set_retry_on_error(value);
	}
}

/// View that fetches events emitted by a block, optionally filtered by extrinsic.
pub struct BlockEvents {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl BlockEvents {
	/// Creates an event view for the given block.
	///
	/// No RPC calls are made until [`BlockEvents::ext`] or [`BlockEvents::block`] is awaited.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		BlockEvents { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Returns events emitted by a specific extrinsic index.
	///
	/// # Returns
	/// - `Ok(Some(ExtrinsicEvents))` when events exist for the given index.
	/// - `Ok(None)` when the block contains no events at that index.
	/// - `Err(Error)` when fetching or decoding event data fails.
	pub async fn ext(&self, tx_index: u32) -> Result<Option<ExtrinsicEvents>, Error> {
		let mut events = self
			.block(BlockEventsOptions {
				filter: Some(tx_index.into()),
				enable_encoding: Some(true),
				enable_decoding: Some(false),
			})
			.await?;

		let Some(first) = events.first_mut() else {
			return Ok(None);
		};

		let mut result: Vec<ExtrinsicEvent> = Vec::with_capacity(first.events.len());
		for phase_event in &mut first.events {
			let Some(data) = phase_event.encoded_data.take() else {
				return Err(RpcError::ExpectedData("The node did not return encoded data for this event.".into()).into());
			};

			let ext_event = ExtrinsicEvent {
				index: phase_event.index,
				pallet_id: phase_event.pallet_id,
				variant_id: phase_event.variant_id,
				data,
			};
			result.push(ext_event);
		}

		Ok(Some(ExtrinsicEvents::new(result)))
	}

	/// Fetches events for the block using the given options.
	///
	/// By default encoding is enabled; callers can override this via `opts`.
	pub async fn block(&self, mut opts: BlockEventsOptions) -> Result<Vec<rpc::BlockPhaseEvent>, Error> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		if opts.enable_encoding.is_none() {
			opts.enable_encoding = Some(true);
		}

		self.client
			.chain()
			.retry_on(retry_on_error, None)
			.system_fetch_events(self.block_id.clone(), opts.into())
			.await
	}

	/// Controls retry behaviour for event lookups: `Some(true)` forces retries, `Some(false)` disables
	/// them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}
}

#[derive(Debug, Default, Clone)]
pub struct BlockEventsOptions {
	filter: Option<rpc::EventFilter>,
	enable_encoding: Option<bool>,
	enable_decoding: Option<bool>,
}

impl Into<rpc::EventOpts> for BlockEventsOptions {
	fn into(self) -> rpc::EventOpts {
		rpc::EventOpts {
			filter: self.filter,
			enable_encoding: self.enable_encoding,
			enable_decoding: self.enable_decoding,
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct BlockExtOptionsSimple {
	pub filter: Option<ExtrinsicFilter>,
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct BlockExtOptionsExpanded {
	pub filter: Option<ExtrinsicFilter>,
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
	pub encode_as: Option<EncodeSelector>,
}

impl Into<rpc::ExtrinsicOpts> for BlockExtOptionsExpanded {
	fn into(self) -> rpc::ExtrinsicOpts {
		rpc::ExtrinsicOpts {
			transaction_filter: self.filter.unwrap_or_default(),
			ss58_address: self.ss58_address,
			app_id: self.app_id,
			nonce: self.nonce,
			encode_as: self.encode_as.unwrap_or_default(),
		}
	}
}

impl From<BlockExtOptionsSimple> for BlockExtOptionsExpanded {
	fn from(value: BlockExtOptionsSimple) -> Self {
		Self {
			filter: value.filter,
			ss58_address: value.ss58_address,
			app_id: value.app_id,
			nonce: value.nonce,
			encode_as: Some(EncodeSelector::Extrinsic),
		}
	}
}

#[derive(Debug, Clone)]
pub struct BlockExtrinsicMetadata {
	pub ext_hash: H256,
	pub ext_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub block_id: HashNumber,
}

impl BlockExtrinsicMetadata {
	/// Wraps metadata about an extrinsic inside a block.
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, block_id: HashNumber) -> Self {
		Self { ext_hash, ext_index, pallet_id, variant_id, block_id }
	}
}

#[derive(Debug, Clone)]
pub struct BlockRawExtrinsic {
	pub data: Option<String>,
	pub metadata: BlockExtrinsicMetadata,
	pub signer_payload: Option<SignerPayload>,
}

impl BlockRawExtrinsic {
	/// Creates a raw extrinsic wrapper.
	pub fn new(data: Option<String>, metadata: BlockExtrinsicMetadata, signer_payload: Option<SignerPayload>) -> Self {
		Self { data, metadata, signer_payload }
	}

	/// Fetches events emitted by this extrinsic.
	///
	/// # Returns
	/// - `Ok(ExtrinsicEvents)` when the block exposes matching events.
	/// - `Err(Error)` when the extrinsic emitted no events or the RPC layer fails.
	pub async fn events(&self, client: Client) -> Result<ExtrinsicEvents, Error> {
		let events = BlockEvents::new(client, self.metadata.block_id.clone())
			.ext(self.ext_index())
			.await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	/// Returns the index of this extrinsic inside the block.
	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	/// Returns the extrinsic hash.
	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	/// Returns the application id if the signer payload provided it.
	pub fn app_id(&self) -> Option<u32> {
		Some(self.signer_payload.as_ref()?.app_id)
	}

	/// Returns the nonce if the signer payload provided it.
	pub fn nonce(&self) -> Option<u32> {
		Some(self.signer_payload.as_ref()?.nonce)
	}

	/// Returns the ss58 address if the signer payload provided it.
	pub fn ss58_address(&self) -> Option<String> {
		self.signer_payload.as_ref()?.ss58_address.clone()
	}
}

/// Decoded extrinsic along with metadata and optional signature.
#[derive(Debug, Clone)]
pub struct BlockExtrinsic<T: HasHeader + Decode> {
	pub signature: Option<ExtrinsicSignature>,
	pub call: T,
	pub metadata: BlockExtrinsicMetadata,
}

impl<T: HasHeader + Decode> BlockExtrinsic<T> {
	/// Creates an extrinsic wrapper from decoded data.
	pub fn new(signature: Option<ExtrinsicSignature>, call: T, metadata: BlockExtrinsicMetadata) -> Self {
		Self { signature, call, metadata }
	}

	/// Fetches events emitted by this extrinsic.
	pub async fn events(&self, client: Client) -> Result<ExtrinsicEvents, Error> {
		let events = BlockEvents::new(client, self.metadata.block_id.clone())
			.ext(self.ext_index())
			.await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for extrinsic".into()).into());
		};

		Ok(events)
	}

	/// Returns the index of this extrinsic inside the block.
	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	/// Returns the extrinsic hash.
	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	/// Returns the application id if the extrinsic was signed.
	pub fn app_id(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.tx_extra.app_id)
	}

	/// Returns the nonce if the extrinsic was signed.
	pub fn nonce(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.tx_extra.nonce)
	}

	/// Returns the tip if the extrinsic was signed.
	pub fn tip(&self) -> Option<u128> {
		Some(self.signature.as_ref()?.tx_extra.tip)
	}

	/// Returns the signer as an ss58 string when available.
	pub fn ss58_address(&self) -> Option<String> {
		match &self.signature.as_ref()?.address {
			MultiAddress::Id(x) => Some(std::format!("{}", x)),
			_ => None,
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockRawExtrinsic> for BlockExtrinsic<T> {
	type Error = String;

	fn try_from(value: BlockRawExtrinsic) -> Result<Self, Self::Error> {
		let Some(data) = &value.data else {
			return Err("Encoded extrinsic payload is missing from the RPC response.")?;
		};

		let extrinsic = Extrinsic::<T>::try_from(data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata))
	}
}

/// Block Transaction is the same as Block Signed Extrinsic
#[derive(Debug, Clone)]
pub struct BlockTransaction<T: HasHeader + Decode> {
	pub signature: ExtrinsicSignature,
	pub call: T,
	pub metadata: BlockExtrinsicMetadata,
}

impl<T: HasHeader + Decode> BlockTransaction<T> {
	/// Creates a transaction wrapper from decoded data.
	pub fn new(signature: ExtrinsicSignature, call: T, metadata: BlockExtrinsicMetadata) -> Self {
		Self { signature, call, metadata }
	}

	/// Fetches events emitted by this transaction.
	pub async fn events(&self, client: Client) -> Result<ExtrinsicEvents, Error> {
		let events = BlockEvents::new(client, self.metadata.block_id)
			.ext(self.ext_index())
			.await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	/// Returns the index of this transaction inside the block.
	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	/// Returns the transaction hash.
	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	/// Returns the application id for this transaction.
	pub fn app_id(&self) -> u32 {
		self.signature.tx_extra.app_id
	}

	/// Returns the signer nonce for this transaction.
	pub fn nonce(&self) -> u32 {
		self.signature.tx_extra.nonce
	}

	/// Returns the paid tip for this transaction.
	pub fn tip(&self) -> u128 {
		self.signature.tx_extra.tip
	}

	/// Returns the signer as an ss58 string when available.
	pub fn ss58_address(&self) -> Option<String> {
		match &self.signature.address {
			MultiAddress::Id(x) => Some(std::format!("{}", x)),
			_ => None,
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockExtrinsic<T>> for BlockTransaction<T> {
	type Error = String;

	fn try_from(value: BlockExtrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature, value.call, value.metadata))
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockRawExtrinsic> for BlockTransaction<T> {
	type Error = String;

	fn try_from(value: BlockRawExtrinsic) -> Result<Self, Self::Error> {
		let ext = BlockExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

#[derive(Debug, Clone)]
pub struct ExtrinsicEvent {
	pub index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub data: String,
}

#[derive(Debug, Clone)]
pub struct ExtrinsicEvents {
	pub events: Vec<ExtrinsicEvent>,
}

impl ExtrinsicEvents {
	/// Wraps decoded events.
	pub fn new(events: Vec<ExtrinsicEvent>) -> Self {
		Self { events }
	}

	/// Returns the first event matching the requested type.
	pub fn first<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns every event matching the requested type.
	pub fn all<T: HasHeader + codec::Decode>(&self) -> Result<Vec<T>, String> {
		let mut result = Vec::new();
		for event in &self.events {
			if event.pallet_id != T::HEADER_INDEX.0 || event.variant_id != T::HEADER_INDEX.1 {
				continue;
			}

			let decoded = T::from_event(event.data.as_str())?;
			result.push(decoded);
		}

		Ok(result)
	}

	/// Checks if an `ExtrinsicSuccess` event exists.
	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	/// Checks if an `ExtrinsicFailed` event exists.
	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	/// Returns whether a proxy call succeeded, when present.
	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::proxy::events::ProxyExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns whether a multisig call succeeded, when present.
	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::multisig::events::MultisigExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns true when at least one event of the given type exists.
	pub fn is_present<T: HasHeader>(&self) -> bool {
		self.count::<T>() > 0
	}

	/// Returns true when the given pallet and variant combination appears.
	pub fn is_present_parts(&self, pallet_id: u8, variant_id: u8) -> bool {
		self.count_parts(pallet_id, variant_id) > 0
	}

	/// Counts how many times the given event type appears.
	pub fn count<T: HasHeader>(&self) -> u32 {
		self.count_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1)
	}

	/// Counts how many events match the pallet and variant combo.
	pub fn count_parts(&self, pallet_id: u8, variant_id: u8) -> u32 {
		let mut count = 0;
		self.events.iter().for_each(|x| {
			if x.pallet_id == pallet_id && x.variant_id == variant_id {
				count += 1
			}
		});

		count
	}
}

fn should_retry(client: &Client, value: Option<bool>) -> bool {
	value.unwrap_or(client.is_global_retries_enabled())
}
