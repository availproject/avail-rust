use crate::{
	Client, Error, UserError,
	block::{
		encoded::{BlockEncodedExtrinsic, BlockEncodedExtrinsics, ExtrinsicsOpts, Metadata},
		events::{AllEvents, Events},
		signed::BlockSignedExtrinsic,
	},
};
use avail_rust_core::{
	Extrinsic as CoreExtrinsic, ExtrinsicSignature, H256, HasHeader, MultiAddress, RpcError, rpc::ExtrinsicFilter,
	types::HashStringNumber,
};
use codec::Decode;

/// View of block extrinsics decoded into calls and optional signatures.
pub struct BlockExtrinsics {
	xt: BlockEncodedExtrinsics,
}

impl BlockExtrinsics {
	/// Builds a decoded extrinsic view for the specified block.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch encoded extrinsics before decoding.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Helper that decodes extrinsics on demand.
	pub fn new(client: Client, block_id: HashStringNumber) -> Self {
		Self { xt: BlockEncodedExtrinsics::new(client, block_id) }
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
	) -> Result<Option<BlockExtrinsic<T>>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &BlockExtrinsics,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<BlockExtrinsic<T>>, Error> {
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
	pub async fn first<T: HasHeader + Decode>(
		&self,
		mut opts: ExtrinsicsOpts,
	) -> Result<Option<BlockExtrinsic<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let first = self.xt.first(opts).await?;
		let Some(first) = first else {
			return Ok(None);
		};

		let ext = CoreExtrinsic::<T>::try_from(first.data.as_str()).map_err(UserError::Decoding)?;
		let ext = BlockExtrinsic::new(ext.signature, ext.call, first.metadata);

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
	pub async fn last<T: HasHeader + Decode>(
		&self,
		mut opts: ExtrinsicsOpts,
	) -> Result<Option<BlockExtrinsic<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let last = self.xt.last(opts).await?;
		let Some(last) = last else {
			return Ok(None);
		};

		let ext = CoreExtrinsic::<T>::try_from(last.data.as_str()).map_err(UserError::Decoding)?;
		let ext = BlockExtrinsic::new(ext.signature, ext.call, last.metadata);
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
	pub async fn all<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Vec<BlockExtrinsic<T>>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		let all = self.xt.all(opts).await?;
		let mut result = Vec::with_capacity(all.len());
		for raw_ext in all {
			let ext = CoreExtrinsic::<T>::try_from(raw_ext.data.as_str()).map_err(UserError::Decoding)?;
			let ext = BlockExtrinsic::new(ext.signature, ext.call, raw_ext.metadata);
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
}

/// Decoded extrinsic along with metadata and optional signature.
#[derive(Debug, Clone)]
pub struct BlockExtrinsic<T: HasHeader + Decode> {
	/// Optional signature associated with the extrinsic.
	pub signature: Option<ExtrinsicSignature>,
	/// Decoded runtime call payload.
	pub call: T,
	/// Metadata describing where the extrinsic was found.
	pub metadata: Metadata,
}

impl<T: HasHeader + Decode> BlockExtrinsic<T> {
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
	pub fn as_signed(&self) -> Result<BlockSignedExtrinsic<T>, String>
	where
		T: Clone,
	{
		BlockSignedExtrinsic::<T>::try_from(self)
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockEncodedExtrinsic> for BlockExtrinsic<T> {
	type Error = String;

	/// Decodes an encoded extrinsic into an `Extrinsic<T>`.
	///
	/// # Parameters
	/// - `value`: Encoded extrinsic containing the SCALE payload.
	///
	/// # Returns
	/// - `Ok(Self)`: Decoded extrinsic with optional signature and metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	fn try_from(value: BlockEncodedExtrinsic) -> Result<Self, Self::Error> {
		let extrinsic = CoreExtrinsic::<T>::try_from(value.data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata))
	}
}

impl<T: HasHeader + Decode> TryFrom<&BlockEncodedExtrinsic> for BlockExtrinsic<T> {
	type Error = String;

	/// Decodes a borrowed encoded extrinsic into an `Extrinsic<T>`.
	///
	/// # Parameters
	/// - `value`: Borrowed encoded extrinsic containing the SCALE payload.
	///
	/// # Returns
	/// - `Ok(Self)`: Decoded extrinsic with optional signature and cloned metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	fn try_from(value: &BlockEncodedExtrinsic) -> Result<Self, Self::Error> {
		let extrinsic = CoreExtrinsic::<T>::try_from(value.data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata.clone()))
	}
}
