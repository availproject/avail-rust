use crate::{
	Client, Error,
	block::{
		BlockExtrinsicMetadata,
		encoded::BlockEncodedExtrinsic,
		events::{BlockEvents, BlockEventsQuery},
		extrinsic::BlockExtrinsic,
	},
};
use avail_rust_core::{ExtrinsicSignature, H256, HasHeader, MultiAddress, RpcError};
use codec::Decode;

// /// View of block extrinsics restricted to signed transactions.
// pub struct BlockSignedExtrinsicsQuery {
// 	xt: BlockExtrinsicsQuery,
// }

// impl BlockSignedExtrinsicsQuery {
// 	/// Builds a signed-transaction view for the specified block.
// 	///
// 	/// # Parameters
// 	/// - `client`: RPC client used to access extrinsic data.
// 	/// - `block_id`: Identifier convertible into `HashStringNumber`.
// 	///
// 	/// # Returns
// 	/// - `Self`: Helper that only surfaces signed extrinsics.
// 	pub fn new(client: Client, block_id: HashStringNumber) -> Self {
// 		Self { xt: BlockExtrinsicsQuery::new(client, block_id) }
// 	}

// 	/// Fetches a signed transaction by hash, index, or string identifier.
// 	///
// 	/// # Parameters
// 	/// - `extrinsic_id`: Identifier used to select the extrinsic.
// 	///
// 	/// # Returns
// 	/// - `Ok(Some(SignedExtrinsic<T>))`: Matching extrinsic decoded as `T` with a signature.
// 	/// - `Ok(None)`: No extrinsic matched the identifier.
// 	/// - `Err(Error)`: The extrinsic was unsigned, or the RPC call/decoding failed.
// 	///
// 	/// # Side Effects
// 	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
// 	pub async fn get<T: HasHeader + Decode>(
// 		&self,
// 		extrinsic_id: impl Into<HashStringNumber>,
// 	) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
// 		let ext = self.xt.get(extrinsic_id).await?;
// 		let Some(ext) = ext else {
// 			return Ok(None);
// 		};

// 		Ok(Some(ext.as_signed()?))
// 	}

// 	/// Returns the first signed extrinsic that matches the supplied filters.
// 	///
// 	/// # Parameters
// 	/// - `opts`: Filters describing which signed extrinsics to fetch.
// 	///
// 	/// # Returns
// 	/// - `Ok(Some(SignedExtrinsic<T>))`: First matching signed extrinsic decoded as `T`.
// 	/// - `Ok(None)`: No signed extrinsic satisfied the filters.
// 	/// - `Err(Error)`: The extrinsic was unsigned, or the RPC call/decoding failed.
// 	///
// 	/// # Side Effects
// 	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
// 	pub async fn first<T: HasHeader + Decode>(&self, opts: Options) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
// 		let ext = self.xt.first(opts).await?;
// 		let Some(ext) = ext else {
// 			return Ok(None);
// 		};

// 		Ok(Some(ext.as_signed()?))
// 	}

// 	/// Returns the last signed extrinsic that matches the supplied filters.
// 	///
// 	/// # Parameters
// 	/// - `opts`: Filters describing which signed extrinsics to fetch.
// 	///
// 	/// # Returns
// 	/// - `Ok(Some(SignedExtrinsic<T>))`: Final matching signed extrinsic decoded as `T`.
// 	/// - `Ok(None)`: No signed extrinsic satisfied the filters.
// 	/// - `Err(Error)`: The extrinsic was unsigned, or the RPC call/decoding failed.
// 	///
// 	/// # Side Effects
// 	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
// 	pub async fn last<T: HasHeader + Decode>(&self, opts: Options) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
// 		let ext = self.xt.last(opts).await?;
// 		let Some(ext) = ext else {
// 			return Ok(None);
// 		};

// 		Ok(Some(ext.as_signed()?))
// 	}

// 	/// Collects every signed extrinsic that matches the supplied filters.
// 	///
// 	/// # Parameters
// 	/// - `opts`: Filters describing which signed extrinsics to fetch.
// 	///
// 	/// # Returns
// 	/// - `Ok(Vec<SignedExtrinsic<T>>)`: Zero or more signed extrinsics decoded as `T`.
// 	/// - `Err(Error)`: An extrinsic lacked a signature, or the RPC call/decoding failed.
// 	///
// 	/// # Side Effects
// 	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
// 	pub async fn all<T: HasHeader + Decode>(&self, opts: Options) -> Result<Vec<BlockSignedExtrinsic<T>>, Error> {
// 		let all = self.xt.all::<T>(opts).await?;
// 		let mut result = Vec::with_capacity(all.len());
// 		for ext in all {
// 			let Some(signature) = ext.signature else {
// 				return Err(UserError::Other(
// 					"Extrinsic is unsigned; cannot decode it as a signed transaction.".into(),
// 				)
// 				.into());
// 			};
// 			result.push(BlockSignedExtrinsic::new(signature, ext.call, ext.metadata));
// 		}

// 		Ok(result)
// 	}

// 	/// Counts matching signed extrinsics.
// 	///
// 	/// # Parameters
// 	/// - `opts`: Filters describing which signed extrinsics to count.
// 	///
// 	/// # Returns
// 	/// - `Ok(usize)`: Number of matching signed extrinsics.
// 	/// - `Err(Error)`: The RPC call failed.
// 	///
// 	/// # Side Effects
// 	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
// 	pub async fn count<T: HasHeader>(&self, opts: Options) -> Result<usize, Error> {
// 		self.xt.count::<T>(opts).await
// 	}

// 	/// Reports whether any signed extrinsic matches the supplied filters.
// 	///
// 	/// # Parameters
// 	/// - `opts`: Filters describing which signed extrinsics to test.
// 	///
// 	/// # Returns
// 	/// - `Ok(true)`: At least one matching signed extrinsic exists.
// 	/// - `Ok(false)`: No signed extrinsics matched the filters.
// 	/// - `Err(Error)`: The RPC call failed.
// 	///
// 	/// # Side Effects
// 	/// - Performs RPC calls via the decoded-extrinsic helper and may retry according to the retry policy.
// 	pub async fn exists<T: HasHeader>(&self, opts: Options) -> Result<bool, Error> {
// 		self.xt.exists::<T>(opts).await
// 	}

// 	/// Overrides the retry behaviour for future signed-transaction lookups.
// 	///
// 	/// # Parameters
// 	/// - `value`: `Some(true)` to force retries, `Some(false)` to disable retries, `None` to inherit the client default.
// 	///
// 	/// # Returns
// 	/// - `()`: The override is stored for subsequent operations.
// 	///
// 	/// # Side Effects
// 	/// - Updates the internal retry setting used by follow-up RPC calls.
// 	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
// 		self.xt.set_retry_on_error(value);
// 	}

// 	/// Reports whether signed-transaction lookups retry after RPC errors.
// 	///
// 	/// # Returns
// 	/// - `true`: Retries are enabled either explicitly or via the client default.
// 	/// - `false`: Retries are disabled.
// 	pub fn should_retry_on_error(&self) -> bool {
// 		self.xt.should_retry_on_error()
// 	}
// }

/// Block Transaction is the same as Block Signed Extrinsic
#[derive(Debug, Clone)]
pub struct BlockSignedExtrinsic<T: HasHeader + Decode> {
	/// Signature proving authorship of the extrinsic.
	pub signature: ExtrinsicSignature,
	/// Decoded runtime call payload.
	pub call: T,
	/// Metadata describing where the extrinsic was found.
	pub metadata: BlockExtrinsicMetadata,
}

impl<T: HasHeader + Decode> BlockSignedExtrinsic<T> {
	/// Creates a transaction wrapper from decoded data.
	///
	/// # Parameters
	/// - `signature`: Signature associated with the extrinsic.
	/// - `call`: Decoded call payload.
	/// - `metadata`: Metadata describing the extrinsic context.
	///
	/// # Returns
	/// - `Self`: Signed extrinsic wrapper containing the provided data.
	pub fn new(signature: ExtrinsicSignature, call: T, metadata: BlockExtrinsicMetadata) -> Self {
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
	pub async fn events(&self, client: Client) -> Result<BlockEvents, Error> {
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

	/// Returns the application id for this transaction.
	///
	/// # Returns
	/// - `u32`: Application identifier recorded in the signature.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn app_id(&self) -> u32 {
		self.signature.extra.app_id
	}

	/// Returns the signer nonce for this transaction.
	///
	/// # Returns
	/// - `u32`: Nonce recorded in the signature.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn nonce(&self) -> u32 {
		self.signature.extra.nonce
	}

	/// Returns the paid tip for this transaction.
	///
	/// # Returns
	/// - `u128`: Tip recorded in the signature.
	///
	/// # Side Effects
	/// - None; reads cached signature information.
	pub fn tip(&self) -> u128 {
		self.signature.extra.tip
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

impl<T: HasHeader + Decode> TryFrom<BlockExtrinsic<T>> for BlockSignedExtrinsic<T> {
	type Error = String;

	/// Converts a decoded extrinsic into a signed extrinsic when a signature is present.
	///
	/// # Parameters
	/// - `value`: Decoded extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic carrying the call and metadata.
	/// - `Err(String)`: The extrinsic was unsigned.
	fn try_from(value: BlockExtrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature, value.call, value.metadata))
	}
}

impl<T: HasHeader + Decode + Clone> TryFrom<&BlockExtrinsic<T>> for BlockSignedExtrinsic<T> {
	type Error = String;

	/// Converts a borrowed extrinsic into a signed extrinsic when a signature is present.
	///
	/// # Parameters
	/// - `value`: Borrowed extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic with cloned call and metadata.
	/// - `Err(String)`: The extrinsic was unsigned.
	fn try_from(value: &BlockExtrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = &value.signature else {
			return Err("Extrinsic is unsigned; expected a signature.")?;
		};

		Ok(Self::new(signature.clone(), value.call.clone(), value.metadata.clone()))
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockEncodedExtrinsic> for BlockSignedExtrinsic<T> {
	type Error = String;

	/// Decodes an encoded extrinsic into a signed extrinsic.
	///
	/// # Parameters
	/// - `value`: Encoded extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic decoded from the payload.
	/// - `Err(String)`: Decoding failed or the extrinsic was unsigned.
	fn try_from(value: BlockEncodedExtrinsic) -> Result<Self, Self::Error> {
		let ext = BlockExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<&BlockEncodedExtrinsic> for BlockSignedExtrinsic<T> {
	type Error = String;

	/// Decodes a borrowed encoded extrinsic into a signed extrinsic.
	///
	/// # Parameters
	/// - `value`: Borrowed encoded extrinsic expected to contain a signature.
	///
	/// # Returns
	/// - `Ok(Self)`: Signed extrinsic decoded from the payload.
	/// - `Err(String)`: Decoding failed or the extrinsic was unsigned.
	fn try_from(value: &BlockEncodedExtrinsic) -> Result<Self, Self::Error> {
		let ext = BlockExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}
