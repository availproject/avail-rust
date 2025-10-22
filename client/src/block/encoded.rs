use crate::{
	Client, Error, UserError,
	block::{
		calls::ExtrinsicCalls,
		events::{AllEvents, Events},
		extrinsic::BlockExtrinsic,
		shared::BlockContext,
		signed::BlockSignedExtrinsic,
	},
};
use avail_rust_core::{
	EncodeSelector, H256, HasHeader, HashNumber, RpcError, avail,
	rpc::{self, ExtrinsicFilter, SignerPayload},
	types::HashStringNumber,
};
use codec::Decode;

/// View of block extrinsics as raw payloads with associated metadata.
pub struct BlockEncodedExtrinsics {
	ctx: BlockContext,
}

impl BlockEncodedExtrinsics {
	/// Builds a raw extrinsic view for the specified block.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch encoded extrinsics.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Encoded-extrinsic helper scoped to the provided block.
	pub fn new(client: Client, block_id: HashStringNumber) -> Self {
		Self { ctx: BlockContext::new(client, block_id) }
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
	pub async fn get(&self, extrinsic_id: impl Into<HashStringNumber>) -> Result<Option<BlockEncodedExtrinsic>, Error> {
		async fn inner(
			s: &BlockEncodedExtrinsics,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<BlockEncodedExtrinsic>, Error> {
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
	pub async fn first(&self, opts: ExtrinsicsOpts) -> Result<Option<BlockEncodedExtrinsic>, Error> {
		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let Some(data) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let metadata = Metadata::new(first.ext_hash, first.ext_index, first.pallet_id, first.variant_id, block_id);
		let ext = BlockEncodedExtrinsic::new(data, metadata, first.signer_payload.take());

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
	pub async fn last(&self, opts: ExtrinsicsOpts) -> Result<Option<BlockEncodedExtrinsic>, Error> {
		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(last) = result.last_mut() else {
			return Ok(None);
		};

		let Some(data) = last.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let metadata = Metadata::new(last.ext_hash, last.ext_index, last.pallet_id, last.variant_id, block_id);
		let ext = BlockEncodedExtrinsic::new(data, metadata, last.signer_payload.take());

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
	pub async fn all(&self, opts: ExtrinsicsOpts) -> Result<Vec<BlockEncodedExtrinsic>, Error> {
		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let extrinsics = chain.system_fetch_extrinsics(block_id, opts).await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for ext in extrinsics {
			let metadata = Metadata::new(ext.ext_hash, ext.ext_index, ext.pallet_id, ext.variant_id, block_id);
			let Some(data) = ext.data else {
				return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
			};

			let enc_ext = BlockEncodedExtrinsic::new(data, metadata, ext.signer_payload);
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

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
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
		self.ctx.set_retry_on_error(value);
	}

	/// Reports whether encoded-extrinsic lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
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
		let calls = ExtrinsicCalls::new(self.ctx.client.clone(), self.ctx.block_id.clone());

		let timestamp = calls.first::<avail::timestamp::tx::Set>(Default::default()).await?;
		let Some(timestamp) = timestamp else {
			return Err(Error::User(UserError::Other(std::format!(
				"No timestamp transaction found in block: {:?}",
				self.ctx.block_id
			))));
		};

		Ok(timestamp.now)
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
}

#[derive(Debug, Clone)]
pub struct BlockEncodedExtrinsic {
	/// SCALE-encoded string representation of the extrinsic.
	pub data: String,
	/// Associated metadata describing the extrinsic.
	pub metadata: Metadata,
	/// Optional signer payload supplied by the node.
	pub signer_payload: Option<SignerPayload>,
}

impl BlockEncodedExtrinsic {
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
	pub fn as_signed<T: HasHeader + Decode>(&self) -> Result<BlockSignedExtrinsic<T>, String> {
		BlockSignedExtrinsic::<T>::try_from(self)
	}

	/// Converts the encoded extrinsic into a decoded extrinsic wrapper.
	///
	/// # Returns
	/// - `Ok(Extrinsic<T>)`: Decoded extrinsic containing the call and metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	pub fn as_extrinsic<T: HasHeader + Decode>(&self) -> Result<BlockExtrinsic<T>, String> {
		BlockExtrinsic::<T>::try_from(self)
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
