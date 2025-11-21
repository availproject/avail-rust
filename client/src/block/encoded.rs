use crate::{
	Client, Error, UserError,
	block::{
		BlockExtrinsicMetadata,
		events::{BlockEvents, BlockEventsQuery},
		extrinsic::BlockExtrinsic,
		extrinsic_options::Options,
		shared::BlockContext,
		signed::BlockSignedExtrinsic,
	},
};
use avail_rust_core::{
	EncodeSelector, EncodedExtrinsic, ExtrinsicSignature, H256, HasHeader, HashNumber, RpcError,
	rpc::{self, ExtrinsicFilter, ExtrinsicInfo},
	types::HashStringNumber,
};
use codec::Decode;

/// View of block extrinsics as raw payloads with associated metadata.
pub struct BlockEncodedExtrinsicsQuery {
	ctx: BlockContext,
}

impl BlockEncodedExtrinsicsQuery {
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
			s: &BlockEncodedExtrinsicsQuery,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<BlockEncodedExtrinsic>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			let opts = Options::new().filter(filter);

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
	pub async fn first(&self, opts: Options) -> Result<Option<BlockEncodedExtrinsic>, Error> {
		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(info) = result.first_mut() else {
			return Ok(None);
		};

		let ext = BlockEncodedExtrinsic::from_extrinsic_info(info, block_id)?;
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
	pub async fn last(&self, opts: Options) -> Result<Option<BlockEncodedExtrinsic>, Error> {
		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(info) = result.last_mut() else {
			return Ok(None);
		};

		let ext = BlockEncodedExtrinsic::from_extrinsic_info(info, block_id)?;
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
	pub async fn all(&self, opts: Options) -> Result<Vec<BlockEncodedExtrinsic>, Error> {
		let block_id = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let opts = opts.to_rpc_opts(EncodeSelector::Extrinsic);
		let extrinsics = chain.system_fetch_extrinsics(block_id, opts).await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for info in extrinsics {
			let ext = BlockEncodedExtrinsic::from_extrinsic_info(&info, block_id)?;
			result.push(ext);
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
	pub async fn count(&self, opts: Options) -> Result<usize, Error> {
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
	/// - Performs an RPC call via [`Self::count`] and may retry according to the retry policy.
	pub async fn exists(&self, opts: Options) -> Result<bool, Error> {
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
}

/// Encoded extrinsic payload paired with signature and metadata helpers.
#[derive(Debug, Clone)]
pub struct BlockEncodedExtrinsic {
	/// Optional signature associated with the extrinsic.
	pub signature: Option<ExtrinsicSignature>,
	/// Encoded runtime call payload.
	pub call: Vec<u8>,
	/// Metadata describing where the extrinsic was found.
	pub metadata: BlockExtrinsicMetadata,
}

impl BlockEncodedExtrinsic {
	/// Creates an encoded extrinsic wrapper.
	///
	/// # Arguments
	/// * `signature` - Optional signature shipped alongside the payload.
	/// * `call` - SCALE-encoded call bytes.
	/// * `metadata` - Metadata identifying where the extrinsic resides.
	///
	/// # Returns
	/// Returns a wrapper that exposes convenience accessors.
	pub fn new(signature: Option<ExtrinsicSignature>, call: Vec<u8>, metadata: BlockExtrinsicMetadata) -> Self {
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
	pub async fn events(&self, client: Client) -> Result<BlockEvents, Error> {
		let events = BlockEventsQuery::new(client, self.metadata.block_id)
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

	/// Returns the nonce if the signer payload provided it.
	///
	/// # Returns
	/// - `Some(u32)`: Nonce from the signer payload.
	/// - `None`: Signer payload was absent.
	pub fn nonce(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.extra.nonce)
	}

	/// Returns the tip if the extrinsic was signed.
	///
	/// # Returns
	/// - `Some(u128)`: Tip reported by the signature.
	/// - `None`: Extrinsic was unsigned.
	pub fn tip(&self) -> Option<u128> {
		Some(self.signature.as_ref()?.extra.tip)
	}

	/// Returns the ss58 address if the signer payload provided it.
	///
	/// # Returns
	/// - `Some(String)`: SS58 address supplied by the signer payload.
	/// - `None`: Signer payload was absent.
	pub fn ss58_address(&self) -> Option<String> {
		match &self.signature.as_ref()?.address {
			avail_rust_core::MultiAddress::Id(account_id32) => Some(std::format!("{}", account_id32)),
			_ => None,
		}
	}

	/// Converts the encoded extrinsic into a decoded extrinsic wrapper.
	///
	/// # Returns
	/// - `Ok(Extrinsic<T>)`: Decoded extrinsic containing the call and metadata.
	/// - `Err(String)`: Payload failed to decode as `T`.
	pub fn as_extrinsic<T: HasHeader + Decode>(self) -> Result<BlockExtrinsic<T>, Error> {
		BlockExtrinsic::<T>::try_from(self).map_err(Error::Other)
	}

	/// Converts the encoded extrinsic into a signed variant when possible.
	///
	/// # Returns
	/// - `Ok(SignedExtrinsic<T>)`: Signed extrinsic decoded from the encoded payload.
	/// - `Err(String)`: The extrinsic was unsigned or failed to decode as `T`.
	pub fn as_signed<T: HasHeader + Decode>(self) -> Result<BlockSignedExtrinsic<T>, Error> {
		BlockSignedExtrinsic::<T>::try_from(self).map_err(Error::Other)
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

	/// Constructs an encoded extrinsic wrapper from RPC metadata.
	///
	/// # Arguments
	/// * `info` - RPC response describing the extrinsic.
	/// * `block_id` - Block identifier in which the extrinsic resides.
	///
	/// # Returns
	/// Returns the encoded extrinsic wrapper or an error if payload data was missing or invalid.
	pub fn from_extrinsic_info(info: &ExtrinsicInfo, block_id: HashNumber) -> Result<Self, Error> {
		let metadata = BlockExtrinsicMetadata::from_extrinsic_info(info, block_id);
		let Some(data) = info.data.as_ref() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let extrinsic = EncodedExtrinsic::try_from(data).map_err(Error::Other)?;
		Ok(BlockEncodedExtrinsic::new(extrinsic.signature, extrinsic.call, metadata))
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use crate::TURING_ENDPOINT;
	use avail_rust_core::{ExtrinsicDecodable, avail};

	fn match_timestamp(ext: &BlockEncodedExtrinsic) {
		assert_eq!(
			std::format!("{:?}", ext.ext_hash()),
			"0xdbfa60611f72a714100338db1c7b11c66636a76f116b214d879de069afe67a74"
		);
		assert_eq!(ext.ext_index(), 0);
		assert_eq!(ext.nonce(), None);
		assert_eq!(ext.header(), (3, 0));
		assert!(ext.signature.is_none());
		let set = avail::timestamp::tx::Set::from_call(&ext.call).unwrap();
		assert_eq!(set.now, 1761567760000);
	}

	fn match_failed_send_message(ext: &BlockEncodedExtrinsic) {
		assert_eq!(
			std::format!("{:?}", ext.ext_hash()),
			"0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218"
		);
		assert_eq!(ext.header(), (39, 11));
		assert_eq!(ext.ext_index(), 3);
		assert_eq!(ext.nonce(), None);
		assert!(ext.signature.is_none());
		let f = avail::vector::tx::FailedSendMessageTxs::from_call(&ext.call).unwrap();
		assert_eq!(f.failed_txs.len(), 0);
	}

	fn match_submit_data_1(ext: &BlockEncodedExtrinsic) {
		assert_eq!(
			std::format!("{:?}", ext.ext_hash()),
			"0x8b84294cba5f2b88e2887ac999ebac3806af7be9cca2a521fc889421f240f3ef"
		);
		assert_eq!(ext.ext_index(), 1);
		assert_eq!(ext.header(), (29, 1));
		assert_eq!(ext.nonce(), Some(30));
		assert!(ext.signature.is_some());
		assert_eq!(ext.ss58_address(), Some("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ".to_string()));
		let sd = avail::data_availability::tx::SubmitData::from_call(&ext.call).unwrap();
		assert_eq!(String::from_utf8(sd.data).unwrap(), "AABBCC");
	}

	fn match_submit_data_2(ext: &BlockEncodedExtrinsic) {
		assert_eq!(
			std::format!("{:?}", ext.ext_hash()),
			"0x19fab0492322016c644af12f1547c587ef51edd10311db85cb3aa2680f6ae4ba"
		);
		assert_eq!(ext.ext_index(), 2);
		assert_eq!(ext.header(), (29, 1));
		assert_eq!(ext.nonce(), Some(4));
		assert!(ext.signature.is_some());
		assert_eq!(ext.ss58_address(), Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk".to_string()));
		let sd = avail::data_availability::tx::SubmitData::from_call(&ext.call).unwrap();
		assert_eq!(String::from_utf8(sd.data).unwrap(), "CCBBAA");
	}

	#[tokio::test]
	async fn query_get_test() {
		let client = Client::new(TURING_ENDPOINT).await.unwrap();
		let query = client.block(2491314).encoded();

		for i in 0..4usize {
			let ext = query.get(i as u32).await.unwrap().unwrap();

			// Content check
			match i {
				0 => match_timestamp(&ext),
				1 => match_submit_data_1(&ext),
				2 => match_submit_data_2(&ext),
				3 => match_failed_send_message(&ext),
				_ => panic!(),
			};
		}

		// Non Existing
		assert!(query.get(4).await.unwrap().is_none());
	}

	#[tokio::test]
	async fn query_first_test() {
		let client = Client::new(TURING_ENDPOINT).await.unwrap();
		let query = client.block(2491314).encoded();

		// App Id 1
		let opts = Options::new();
		let ext = query.first(opts).await.unwrap().unwrap();
		match_submit_data_1(&ext);

		// App Id 2
		let opts = Options::new();
		let ext = query.first(opts).await.unwrap().unwrap();
		match_submit_data_2(&ext);

		// Nonce 30
		let opts = Options::new().nonce(30);
		let ext = query.first(opts).await.unwrap().unwrap();
		match_submit_data_1(&ext);

		// Nonce 4
		let opts = Options::new().nonce(4);
		let ext = query.first(opts).await.unwrap().unwrap();
		match_submit_data_2(&ext);

		// DA call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
		let ext = query.first(opts).await.unwrap().unwrap();
		match_submit_data_1(&ext);

		// Pall Call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX.0);
		let ext = query.first(opts).await.unwrap().unwrap();
		match_submit_data_1(&ext);

		// Nothing
		let ext = query.first(Default::default()).await.unwrap().unwrap();
		match_timestamp(&ext);

		// Non Existing
		let opts = Options::new().filter(100u32);
		assert!(query.first(opts).await.unwrap().is_none());
	}

	#[tokio::test]
	async fn query_last_test() {
		let client = Client::new(TURING_ENDPOINT).await.unwrap();
		let query = client.block(2491314).encoded();

		// App Id 1
		let opts = Options::new();
		let ext = query.last(opts).await.unwrap().unwrap();
		match_submit_data_1(&ext);

		// App Id 2
		let opts = Options::new();
		let ext = query.last(opts).await.unwrap().unwrap();
		match_submit_data_2(&ext);

		// Nonce 30
		let opts = Options::new().nonce(30);
		let ext = query.last(opts).await.unwrap().unwrap();
		match_submit_data_1(&ext);

		// Nonce 4
		let opts = Options::new().nonce(4);
		let ext = query.last(opts).await.unwrap().unwrap();
		match_submit_data_2(&ext);

		// DA call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
		let ext = query.last(opts).await.unwrap().unwrap();
		match_submit_data_2(&ext);

		// Pall Call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX.0);
		let ext = query.last(opts).await.unwrap().unwrap();
		match_submit_data_2(&ext);

		// Nothing
		let ext = query.last(Default::default()).await.unwrap().unwrap();
		match_failed_send_message(&ext);

		// Non Existing
		let opts = Options::new().filter(100u32);
		assert!(query.last(opts).await.unwrap().is_none());
	}

	#[tokio::test]
	async fn query_all_test() {
		let client = Client::new(TURING_ENDPOINT).await.unwrap();
		let query = client.block(2491314).encoded();

		// App Id 1
		let opts = Options::new();
		let ext = query.all(opts).await.unwrap();
		match_submit_data_1(&ext[0]);

		// App Id 2
		let opts = Options::new();
		let ext = query.all(opts).await.unwrap();
		match_submit_data_2(&ext[0]);
		assert_eq!(ext.len(), 1);

		// Nonce 30
		let opts = Options::new().nonce(30);
		let ext = query.all(opts).await.unwrap();
		match_submit_data_1(&ext[0]);
		assert_eq!(ext.len(), 1);

		// Nonce 4
		let opts = Options::new().nonce(4);
		let ext = query.all(opts).await.unwrap();
		match_submit_data_2(&ext[0]);
		assert_eq!(ext.len(), 1);

		// DA call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
		let ext = query.all(opts).await.unwrap();
		match_submit_data_1(&ext[0]);
		match_submit_data_2(&ext[1]);
		assert_eq!(ext.len(), 2);

		// Pall Call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX.0);
		let ext = query.all(opts).await.unwrap();
		match_submit_data_1(&ext[0]);
		match_submit_data_2(&ext[1]);
		assert_eq!(ext.len(), 2);

		// Nothing
		let ext = query.all(Default::default()).await.unwrap();
		match_timestamp(&ext[0]);
		match_submit_data_1(&ext[1]);
		match_submit_data_2(&ext[2]);
		match_failed_send_message(&ext[3]);
		assert_eq!(ext.len(), 4);

		// Non Existing
		let opts = Options::new().filter(100u32);
		let ext = query.all(opts).await.unwrap();
		assert_eq!(ext.len(), 0)
	}

	#[tokio::test]
	async fn query_count_test() {
		let client = Client::new(TURING_ENDPOINT).await.unwrap();
		let query = client.block(2491314).encoded();

		// App Id 1
		let opts = Options::new();
		assert_eq!(query.count(opts).await.unwrap(), 1);

		// App Id 2
		let opts = Options::new();
		assert_eq!(query.count(opts).await.unwrap(), 1);

		// Nonce 30
		let opts = Options::new().nonce(30);
		assert_eq!(query.count(opts).await.unwrap(), 1);

		// Nonce 4
		let opts = Options::new().nonce(4);
		assert_eq!(query.count(opts).await.unwrap(), 1);

		// DA call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
		assert_eq!(query.count(opts).await.unwrap(), 2);

		// Pall Call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX.0);
		assert_eq!(query.count(opts).await.unwrap(), 2);

		// Nothing
		assert_eq!(query.count(Default::default()).await.unwrap(), 4);

		// Non Existing
		let opts = Options::new().filter(100u32);
		assert_eq!(query.count(opts).await.unwrap(), 0);
	}

	#[tokio::test]
	async fn query_exists_test() {
		let client = Client::new(TURING_ENDPOINT).await.unwrap();
		let query = client.block(2491314).encoded();

		// App Id 1
		let opts = Options::new();
		assert_eq!(query.exists(opts).await.unwrap(), true);

		// App Id 2
		let opts = Options::new();
		assert_eq!(query.exists(opts).await.unwrap(), true);

		// Nonce 30
		let opts = Options::new().nonce(30);
		assert_eq!(query.exists(opts).await.unwrap(), true);

		// Nonce 4
		let opts = Options::new().nonce(4);
		assert_eq!(query.exists(opts).await.unwrap(), true);

		// DA call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
		assert_eq!(query.exists(opts).await.unwrap(), true);

		// Pall Call
		let opts = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX.0);
		assert_eq!(query.exists(opts).await.unwrap(), true);

		// Nothing
		assert_eq!(query.exists(Default::default()).await.unwrap(), true);

		// Non Existing
		let opts = Options::new().filter(100u32);
		assert_eq!(query.exists(opts).await.unwrap(), false);
	}
}
