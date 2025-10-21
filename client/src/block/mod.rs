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
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Block { client, block_id: block_id.into(), retry_on_error: None }
	}

	pub fn signed(&self) -> SignedExtrinsics {
		SignedExtrinsics::new(self.client.clone(), self.block_id.clone())
	}

	pub fn extrinsics(&self) -> Extrinsics {
		Extrinsics::new(self.client.clone(), self.block_id.clone())
	}

	pub fn encoded(&self) -> EncodedExtrinsics {
		EncodedExtrinsics::new(self.client.clone(), self.block_id.clone())
	}

	pub fn calls(&self) -> ExtrinsicCalls {
		ExtrinsicCalls::new(self.client.clone(), self.block_id.clone())
	}

	pub async fn raw_extrinsics(&self, opts: rpc::ExtrinsicOpts) -> Result<Vec<rpc::ExtrinsicInfo>, Error> {
		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.system_fetch_extrinsics(block_id, opts).await
	}

	/// Returns a helper for fetching events from this block.
	pub fn events(&self) -> Events {
		Events::new(self.client.clone(), self.block_id.clone())
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

	/// Returns true when this block view will retry failed RPC calls.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.encoded().timestamp().await
	}

	pub async fn info(&self) -> Result<BlockInfo, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_info_from(self.block_id.clone()).await
	}

	pub async fn header(&self) -> Result<AvailHeader, Error> {
		shared::header(&self.client, self.block_id.clone()).await
	}

	pub async fn author(&self) -> Result<AccountId, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_author(self.block_id.clone()).await
	}

	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.encoded().extrinsic_count().await
	}

	pub async fn event_count(&self) -> Result<u32, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_event_count(self.block_id.clone()).await
	}

	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_weight(self.block_id.clone()).await
	}

	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		self.events().extrinsic_weight().await
	}
}

pub struct ExtrinsicCalls {
	client: Client,
	block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl ExtrinsicCalls {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { client, block_id: block_id.into(), retry_on_error: None }
	}

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

	/// Returns the last matching extrinsic decoded into the target type.
	///
	/// Return semantics mirror [`Extrinsics::first`], but the final matching extrinsic is returned.
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

	/// Returns every matching extrinsic decoded into the target type.
	///
	/// The result may be empty if no extrinsics match. Decoding failures are surfaced as `Err(Error)`.
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

	pub async fn count<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		let mut opts: rpc::ExtrinsicOpts = opts.to_rpc_opts(EncodeSelector::None);
		opts.transaction_filter = T::HEADER_INDEX.into();

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let result = chain.system_fetch_extrinsics(block_id, opts).await?;

		Ok(result.len())
	}

	pub async fn exists<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
		self.count::<T>(opts).await.map(|x| x > 0)
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Returns true when decoded extrinsic lookups retry after RPC errors.
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
	/// Builds a raw extrinsic view for the given block.
	///
	/// The `block_id` may be a height or hash; conversions happen lazily when RPCs are executed.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Finds a specific extrinsic and returns it in the requested format.
	///
	/// # Returns
	/// - `Ok(Some(EncodedExtrinsic))` when the extrinsic is found.
	/// - `Ok(None)` when no extrinsic matches the provided identifier.
	/// - `Err(Error)` when decoding the identifier fails or the RPC call errors.
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

	/// Returns the first matching extrinsic, if any.
	///
	/// # Returns
	/// - `Ok(Some(EncodedExtrinsic))` with metadata and optional payload.
	/// - `Ok(None)` when nothing matches the provided filters.
	/// - `Err(Error)` on RPC failures or when the block identifier cannot be decoded.
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

	/// Returns the last matching extrinsic, if any.
	///
	/// Return semantics mirror [`EncodedExtrinsics::first`], but the final matching element is returned.
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

	/// Returns all matching extrinsics.
	///
	/// The resulting vector may be empty when no extrinsics satisfy the filters.
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

	/// Counts matching extrinsics without fetching the payloads.
	///
	/// Equivalent to `self.all(opts).await.map(|v| v.len())` but avoids transferring payload bytes.
	pub async fn count(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		let opts: rpc::ExtrinsicOpts = opts.to_rpc_opts(EncodeSelector::None);

		let block_id: HashNumber = self.block_id.clone().try_into().map_err(UserError::Decoding)?;
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		let result = chain.system_fetch_extrinsics(block_id, opts).await?;

		Ok(result.len())
	}

	/// Checks whether at least one extrinsic matches.
	pub async fn exists(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
		self.count(opts).await.map(|x| x > 0)
	}

	/// Controls retry behaviour for fetching raw extrinsics: `Some(true)` forces retries,
	/// `Some(false)` disables them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Returns true when raw extrinsic lookups retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

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

	pub async fn info(&self) -> Result<BlockInfo, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_info_from(self.block_id.clone()).await
	}

	pub async fn header(&self) -> Result<AvailHeader, Error> {
		shared::header(&self.client, self.block_id.clone()).await
	}

	pub async fn author(&self) -> Result<AccountId, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_author(self.block_id.clone()).await
	}

	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.count(ExtrinsicsOpts::new()).await.map(|x| x as u32)
	}

	pub async fn event_count(&self) -> Result<u32, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_event_count(self.block_id.clone()).await
	}

	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		let chain = self.client.chain().retry_on(self.retry_on_error, None);
		chain.block_weight(self.block_id.clone()).await
	}

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
	/// Builds a decoded extrinsic view for the given block.
	///
	/// Decoding happens lazily as individual queries are made.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { xt: EncodedExtrinsics::new(client, block_id) }
	}

	/// Fetches a specific extrinsic by id.
	///
	/// # Returns
	/// - `Ok(Some(Extrinsic<T>))` when the extrinsic exists and decodes as `T`.
	/// - `Ok(None)` when no extrinsic matches the identifier or filters.
	/// - `Err(Error)` if the RPC call fails or decoding the identifier/payload fails.
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

	/// Returns the first matching extrinsic decoded into the target type.
	///
	/// # Returns
	/// - `Ok(Some(Extrinsic<T>))` when an extrinsic matches the filters.
	/// - `Ok(None)` when nothing matches.
	/// - `Err(Error)` if RPC retrieval fails or decoding the extrinsic as `T` fails.
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

	/// Returns the last matching extrinsic decoded into the target type.
	///
	/// Return semantics mirror [`Extrinsics::first`], but the final matching extrinsic is returned.
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

	/// Returns every matching extrinsic decoded into the target type.
	///
	/// The result may be empty if no extrinsics match. Decoding failures are surfaced as `Err(Error)`.
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

	/// Counts matching extrinsics without decoding the payloads.
	///
	/// This still performs an RPC round-trip but avoids transferring the encoded call data.
	pub async fn count<T: HasHeader>(&self, mut opts: ExtrinsicsOpts) -> Result<usize, Error> {
		opts.filter = Some(T::HEADER_INDEX.into());

		return self.xt.count(opts).await;
	}

	/// Checks whether any extrinsic matches the filters.
	///
	/// Equivalent to calling [`Extrinsics::first`] and testing the result for `Some`.
	pub async fn exists<T: HasHeader>(&self, mut opts: ExtrinsicsOpts) -> Result<bool, Error> {
		opts.filter = Some(T::HEADER_INDEX.into());

		return self.xt.exists(opts).await;
	}

	/// Controls retry behaviour for decoded-extrinsic lookups: `Some(true)` forces retries,
	/// `Some(false)` disables them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.xt.set_retry_on_error(value);
	}

	/// Returns true when decoded extrinsic lookups retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.xt.should_retry_on_error()
	}

	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.xt.timestamp().await
	}

	pub async fn info(&self) -> Result<BlockInfo, Error> {
		self.xt.info().await
	}

	pub async fn header(&self) -> Result<AvailHeader, Error> {
		self.xt.header().await
	}

	pub async fn author(&self) -> Result<AccountId, Error> {
		self.xt.author().await
	}

	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.xt.extrinsic_count().await
	}

	pub async fn event_count(&self) -> Result<u32, Error> {
		self.xt.event_count().await
	}

	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		self.xt.weight().await
	}

	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		self.xt.extrinsic_weight().await
	}
}

/// View of block extrinsics restricted to signed transactions.
pub struct SignedExtrinsics {
	xt: Extrinsics,
}

impl SignedExtrinsics {
	/// Builds a signed transaction view for the given block.
	///
	/// Only signed extrinsics will be surfaced; unsigned extrinsics produce an `Error` when decoding.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { xt: Extrinsics::new(client, block_id) }
	}

	/// Fetches a signed transaction by id.
	///
	/// # Returns
	/// - `Ok(Some(SignedExtrinsic<T>))` when the extrinsic exists and carries a signature.
	/// - `Ok(None)` when no extrinsic matches the identifier.
	/// - `Err(Error)` when the extrinsic exists but is unsigned or cannot be decoded as `T`.
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

	/// Returns the first matching signed transaction.
	///
	/// Unsigned extrinsics encountered during decoding produce an `Error`.
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

	/// Returns the last matching signed transaction.
	///
	/// Return semantics mirror [`SignedExtrinsics::first`], but returns the final matching transaction.
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

	/// Returns every matching signed transaction.
	///
	/// Decoding stops early with an `Error` if any extrinsic lacks a signature or fails to decode as `T`.
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

	/// Counts matching signed transactions.
	pub async fn count<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		self.xt.count::<T>(opts).await
	}

	/// Checks whether any signed transaction matches the filters.
	pub async fn exists<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
		self.xt.exists::<T>(opts).await
	}

	/// Controls retry behaviour for signed-transaction lookups: `Some(true)` forces retries,
	/// `Some(false)` disables them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.xt.set_retry_on_error(value);
	}

	/// Returns true when signed transaction lookups retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.xt.should_retry_on_error()
	}

	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.xt.timestamp().await
	}

	pub async fn info(&self) -> Result<BlockInfo, Error> {
		self.xt.info().await
	}

	pub async fn header(&self) -> Result<AvailHeader, Error> {
		self.xt.header().await
	}

	pub async fn author(&self) -> Result<AccountId, Error> {
		self.xt.author().await
	}

	pub async fn extrinsic_count(&self) -> Result<u32, Error> {
		self.xt.extrinsic_count().await
	}

	pub async fn event_count(&self) -> Result<u32, Error> {
		self.xt.event_count().await
	}

	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		self.xt.weight().await
	}

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
	/// No RPC calls are made until [`BlockEvents::ext`] or [`BlockEvents::block`] is awaited.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Events { client, block_id: block_id.into(), retry_on_error: None }
	}

	/// Returns events emitted by a specific extrinsic index.
	///
	/// # Returns
	/// - `Ok(Some(ExtrinsicEvents))` when events exist for the given index.
	/// - `Ok(None)` when the block contains no events at that index.
	/// - `Err(Error)` when fetching or decoding event data fails.
	pub async fn extrinsic(&self, tx_index: u32) -> Result<AllEvents, Error> {
		let events = self.all(tx_index.into()).await?;
		Ok(AllEvents::new(events))
	}

	pub async fn system(&self) -> Result<AllEvents, Error> {
		let events = self.all(rpc::EventFilter::OnlyNonExtrinsics).await?;
		let events: Vec<Event> = events
			.into_iter()
			.filter(|x| x.phase.extrinsic_index().is_none())
			.collect();

		Ok(AllEvents::new(events))
	}

	/// Fetches all events for the block using the given options.
	///
	/// By default encoding is enabled; callers can override this via `opts`.
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

	pub async fn raw(&self, opts: rpc::EventOpts) -> Result<Vec<BlockPhaseEvent>, Error> {
		let result = self
			.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.system_fetch_events(self.block_id.clone(), opts)
			.await?;

		Ok(result)
	}

	/// Controls retry behaviour for event lookups: `Some(true)` forces retries, `Some(false)` disables
	/// them, and `None` keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Returns true when event queries retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

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
	pub fn new() -> Self {
		Self::default()
	}

	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	pub fn ss58_address(mut self, value: impl Into<String>) -> Self {
		self.ss58_address = Some(value.into());
		self
	}

	pub fn filter(mut self, value: impl Into<ExtrinsicFilter>) -> Self {
		self.filter = Some(value.into());
		self
	}

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
	pub ext_hash: H256,
	pub ext_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub block_id: HashNumber,
}

impl Metadata {
	/// Wraps metadata about an extrinsic inside a block.
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, block_id: HashNumber) -> Self {
		Self { ext_hash, ext_index, pallet_id, variant_id, block_id }
	}
}

#[derive(Debug, Clone)]
pub struct EncodedExtrinsic {
	pub data: String,
	pub metadata: Metadata,
	pub signer_payload: Option<SignerPayload>,
}

impl EncodedExtrinsic {
	/// Creates a raw extrinsic wrapper.
	pub fn new(data: String, metadata: Metadata, signer_payload: Option<SignerPayload>) -> Self {
		Self { data, metadata, signer_payload }
	}

	/// Fetches events emitted by this extrinsic.
	///
	/// # Returns
	/// - `Ok(ExtrinsicEvents)` when the block exposes matching events.
	/// - `Err(Error)` when the extrinsic emitted no events or the RPC layer fails.
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

	pub fn as_signed<T: HasHeader + Decode>(&self) -> Result<SignedExtrinsic<T>, String> {
		SignedExtrinsic::<T>::try_from(self)
	}

	pub fn as_extrinsic<T: HasHeader + Decode>(&self) -> Result<Extrinsic<T>, String> {
		Extrinsic::<T>::try_from(self)
	}

	pub fn is<T: HasHeader>(&self) -> bool {
		self.metadata.pallet_id == T::HEADER_INDEX.0 && self.metadata.variant_id == T::HEADER_INDEX.1
	}

	pub fn header(&self) -> (u8, u8) {
		(self.metadata.pallet_id, self.metadata.variant_id)
	}
}

/// Decoded extrinsic along with metadata and optional signature.
#[derive(Debug, Clone)]
pub struct Extrinsic<T: HasHeader + Decode> {
	pub signature: Option<ExtrinsicSignature>,
	pub call: T,
	pub metadata: Metadata,
}

impl<T: HasHeader + Decode> Extrinsic<T> {
	/// Creates an extrinsic wrapper from decoded data.
	pub fn new(signature: Option<ExtrinsicSignature>, call: T, metadata: Metadata) -> Self {
		Self { signature, call, metadata }
	}

	/// Fetches events emitted by this extrinsic.
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

	pub fn as_signed(&self) -> Result<SignedExtrinsic<T>, String>
	where
		T: Clone,
	{
		SignedExtrinsic::<T>::try_from(self)
	}
}

impl<T: HasHeader + Decode> TryFrom<EncodedExtrinsic> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: EncodedExtrinsic) -> Result<Self, Self::Error> {
		let extrinsic = CoreExtrinsic::<T>::try_from(value.data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata))
	}
}

impl<T: HasHeader + Decode> TryFrom<&EncodedExtrinsic> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: &EncodedExtrinsic) -> Result<Self, Self::Error> {
		let extrinsic = CoreExtrinsic::<T>::try_from(value.data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata.clone()))
	}
}

/// Block Transaction is the same as Block Signed Extrinsic
#[derive(Debug, Clone)]
pub struct SignedExtrinsic<T: HasHeader + Decode> {
	pub signature: ExtrinsicSignature,
	pub call: T,
	pub metadata: Metadata,
}

impl<T: HasHeader + Decode> SignedExtrinsic<T> {
	/// Creates a transaction wrapper from decoded data.
	pub fn new(signature: ExtrinsicSignature, call: T, metadata: Metadata) -> Self {
		Self { signature, call, metadata }
	}

	/// Fetches events emitted by this transaction.
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

impl<T: HasHeader + Decode> TryFrom<Extrinsic<T>> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: Extrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature, value.call, value.metadata))
	}
}

impl<T: HasHeader + Decode + Clone> TryFrom<&Extrinsic<T>> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &Extrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = &value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature.clone(), value.call.clone(), value.metadata.clone()))
	}
}

impl<T: HasHeader + Decode> TryFrom<EncodedExtrinsic> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: EncodedExtrinsic) -> Result<Self, Self::Error> {
		let ext = Extrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<&EncodedExtrinsic> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &EncodedExtrinsic) -> Result<Self, Self::Error> {
		let ext = Extrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

#[derive(Debug, Clone)]
pub struct Event {
	pub phase: RuntimePhase,
	pub index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub data: String,
}

#[derive(Debug, Clone)]
pub struct AllEvents {
	pub events: Vec<Event>,
}

impl AllEvents {
	/// Wraps decoded events.
	pub fn new(events: Vec<Event>) -> Self {
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

	/// Returns the last event matching the requested type.
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

	pub fn len(&self) -> usize {
		self.events.len()
	}

	pub fn is_empty(&self) -> bool {
		self.events.is_empty()
	}
}

mod shared {
	pub use super::*;

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
