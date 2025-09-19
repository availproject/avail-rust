use crate::{Client, Error, UserError};
use avail_rust_core::{
	EncodeSelector, Extrinsic, ExtrinsicSignature, H256, HasHeader, MultiAddress, RpcError, TransactionEventDecodable,
	avail,
	rpc::{self, ExtrinsicFilter, SignerPayload},
	types::HashStringNumber,
};
use codec::Decode;

pub struct Block {
	pub tx: BlockWithTx,
	pub ext: BlockWithExt,
	pub raw_ext: BlockWithRawExt,
	pub event: BlockEvents,
}

impl Block {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		fn inner(client: Client, block_id: HashStringNumber) -> Block {
			Block {
				tx: BlockWithTx::new(client.clone(), block_id.clone()),
				ext: BlockWithExt::new(client.clone(), block_id.clone()),
				raw_ext: BlockWithRawExt::new(client.clone(), block_id.clone()),
				event: BlockEvents::new(client.clone(), block_id.clone()),
			}
		}

		inner(client, block_id.into())
	}
}

pub struct BlockWithRawExt {
	client: Client,
	block_id: HashStringNumber,
}

impl BlockWithRawExt {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { client, block_id: block_id.into() }
	}

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

	pub async fn first(&self, mut opts: BlockExtOptionsExpanded) -> Result<Option<BlockRawExtrinsic>, Error> {
		if opts.encode_as.is_none() {
			opts.encode_as = Some(EncodeSelector::Extrinsic)
		}

		let mut result = self
			.client
			.rpc()
			.system_fetch_extrinsics(self.block_id.clone(), opts.into())
			.await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let metadata = BlockExtrinsicMetadata::new(
			first.ext_hash,
			first.ext_index,
			first.pallet_id,
			first.variant_id,
			self.block_id.clone(),
		);
		let ext = BlockRawExtrinsic::new(first.data.take(), metadata, first.signer_payload.take());

		Ok(Some(ext))
	}

	pub async fn last(&self, mut opts: BlockExtOptionsExpanded) -> Result<Option<BlockRawExtrinsic>, Error> {
		if opts.encode_as.is_none() {
			opts.encode_as = Some(EncodeSelector::Extrinsic)
		}

		let mut result = self
			.client
			.rpc()
			.system_fetch_extrinsics(self.block_id.clone(), opts.into())
			.await?;

		let Some(last) = result.last_mut() else {
			return Ok(None);
		};

		let metadata = BlockExtrinsicMetadata::new(
			last.ext_hash,
			last.ext_index,
			last.pallet_id,
			last.variant_id,
			self.block_id.clone(),
		);
		let ext = BlockRawExtrinsic::new(last.data.take(), metadata, last.signer_payload.take());

		Ok(Some(ext))
	}

	pub async fn all(&self, mut opts: BlockExtOptionsExpanded) -> Result<Vec<BlockRawExtrinsic>, Error> {
		if opts.encode_as.is_none() {
			opts.encode_as = Some(EncodeSelector::Extrinsic)
		}

		let result = self
			.client
			.rpc()
			.system_fetch_extrinsics(self.block_id.clone(), opts.into())
			.await?;

		let result = result
			.into_iter()
			.map(|x| {
				let metadata = BlockExtrinsicMetadata::new(
					x.ext_hash,
					x.ext_index,
					x.pallet_id,
					x.variant_id,
					self.block_id.clone(),
				);
				BlockRawExtrinsic::new(x.data, metadata, x.signer_payload)
			})
			.collect();

		Ok(result)
	}

	pub async fn count(&self, mut opts: BlockExtOptionsExpanded) -> Result<usize, Error> {
		opts.encode_as = Some(EncodeSelector::None);

		let result = self.all(opts).await?;
		Ok(result.len())
	}

	pub async fn exists(&self, mut opts: BlockExtOptionsExpanded) -> Result<bool, Error> {
		opts.encode_as = Some(EncodeSelector::None);

		let result = self.first(opts).await?;
		Ok(result.is_some())
	}
}

pub struct BlockWithExt {
	rxt: BlockWithRawExt,
}

impl BlockWithExt {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { rxt: BlockWithRawExt::new(client, block_id) }
	}

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

	pub async fn count<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<usize, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		opts.encode_as = Some(EncodeSelector::None);
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		return self.rxt.count(opts).await;
	}

	pub async fn exists<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<bool, Error> {
		let mut opts: BlockExtOptionsExpanded = opts.into();
		opts.encode_as = Some(EncodeSelector::None);
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}

		return self.rxt.exists(opts).await;
	}
}

pub struct BlockWithTx {
	ext: BlockWithExt,
}

impl BlockWithTx {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { ext: BlockWithExt::new(client, block_id) }
	}

	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &BlockWithTx,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			let filter = Some(filter);
			Ok(s.first::<T>(BlockExtOptionsSimple { filter, ..Default::default() })
				.await?)
		}

		inner::<T>(self, extrinsic_id.into()).await
	}

	pub async fn first<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
		let ext = self.ext.first(opts).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(UserError::Other("Cannot decode extrinsic as signed as it was not signed".into()).into());
		};

		let ext = BlockSignedExtrinsic::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	pub async fn last<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Option<BlockSignedExtrinsic<T>>, Error> {
		let ext = self.ext.last(opts).await?;
		let Some(ext) = ext else {
			return Ok(None);
		};

		let Some(signature) = ext.signature else {
			return Err(UserError::Other("Cannot decode extrinsic as signed as it was not signed".into()).into());
		};

		let ext = BlockSignedExtrinsic::new(signature, ext.call, ext.metadata);
		Ok(Some(ext))
	}

	pub async fn all<T: HasHeader + Decode>(
		&self,
		opts: BlockExtOptionsSimple,
	) -> Result<Vec<BlockSignedExtrinsic<T>>, Error> {
		let all = self.ext.all::<T>(opts).await?;
		let mut result = Vec::with_capacity(all.len());
		for ext in all {
			let Some(signature) = ext.signature else {
				return Err(UserError::Other("Cannot decode extrinsic as signed as it was not signed".into()).into());
			};
			result.push(BlockSignedExtrinsic::new(signature, ext.call, ext.metadata));
		}

		Ok(result)
	}

	pub async fn count<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<usize, Error> {
		self.ext.count::<T>(opts).await
	}

	pub async fn exists<T: HasHeader>(&self, opts: BlockExtOptionsSimple) -> Result<bool, Error> {
		self.ext.exists::<T>(opts).await
	}
}

pub struct BlockEvents {
	client: Client,
	block_id: HashStringNumber,
}

impl BlockEvents {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		BlockEvents { client, block_id: block_id.into() }
	}

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
				return Err(RpcError::ExpectedData("No data was provided from event".into()).into());
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

	pub async fn block(&self, opts: BlockEventsOptions) -> Result<Vec<rpc::BlockPhaseEvent>, Error> {
		self.client
			.rpc()
			.system_fetch_events(self.block_id.clone(), opts.into())
			.await
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
	pub block_id: HashStringNumber,
}

impl BlockExtrinsicMetadata {
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, block_id: HashStringNumber) -> Self {
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
	pub fn new(data: Option<String>, metadata: BlockExtrinsicMetadata, signer_payload: Option<SignerPayload>) -> Self {
		Self { data, metadata, signer_payload }
	}

	pub async fn events(&self, client: Client) -> Result<ExtrinsicEvents, Error> {
		let events = BlockEvents::new(client, self.metadata.block_id.clone())
			.ext(self.ext_index())
			.await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for extrinsic".into()).into());
		};

		Ok(events)
	}

	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	pub fn app_id(&self) -> Option<u32> {
		Some(self.signer_payload.as_ref()?.app_id)
	}

	pub fn nonce(&self) -> Option<u32> {
		Some(self.signer_payload.as_ref()?.nonce)
	}

	pub fn ss58_address(&self) -> Option<String> {
		self.signer_payload.as_ref()?.ss58_address.clone()
	}
}

#[derive(Debug, Clone)]
pub struct BlockExtrinsic<T: HasHeader + Decode> {
	pub signature: Option<ExtrinsicSignature>,
	pub call: T,
	pub metadata: BlockExtrinsicMetadata,
}

impl<T: HasHeader + Decode> BlockExtrinsic<T> {
	pub fn new(signature: Option<ExtrinsicSignature>, call: T, metadata: BlockExtrinsicMetadata) -> Self {
		Self { signature, call, metadata }
	}

	pub async fn events(&self, client: Client) -> Result<ExtrinsicEvents, Error> {
		let events = BlockEvents::new(client, self.metadata.block_id.clone())
			.ext(self.ext_index())
			.await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for extrinsic".into()).into());
		};

		Ok(events)
	}

	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	pub fn app_id(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.tx_extra.app_id)
	}

	pub fn nonce(&self) -> Option<u32> {
		Some(self.signature.as_ref()?.tx_extra.nonce)
	}

	pub fn tip(&self) -> Option<u128> {
		Some(self.signature.as_ref()?.tx_extra.tip)
	}

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
			return Err("No data found in extrinsic info")?;
		};

		let extrinsic = Extrinsic::<T>::try_from(data.as_str())?;
		Ok(Self::new(extrinsic.signature, extrinsic.call, value.metadata))
	}
}

#[derive(Debug, Clone)]
pub struct BlockSignedExtrinsic<T: HasHeader + Decode> {
	pub signature: ExtrinsicSignature,
	pub call: T,
	pub metadata: BlockExtrinsicMetadata,
}

impl<T: HasHeader + Decode> BlockSignedExtrinsic<T> {
	pub fn new(signature: ExtrinsicSignature, call: T, metadata: BlockExtrinsicMetadata) -> Self {
		Self { signature, call, metadata }
	}

	pub async fn events(&self, client: Client) -> Result<ExtrinsicEvents, Error> {
		let events = BlockEvents::new(client, self.metadata.block_id.clone())
			.ext(self.ext_index())
			.await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for extrinsic".into()).into());
		};

		Ok(events)
	}

	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	pub fn app_id(&self) -> u32 {
		self.signature.tx_extra.app_id
	}

	pub fn nonce(&self) -> u32 {
		self.signature.tx_extra.nonce
	}

	pub fn tip(&self) -> u128 {
		self.signature.tx_extra.tip
	}

	pub fn ss58_address(&self) -> Option<String> {
		match &self.signature.address {
			MultiAddress::Id(x) => Some(std::format!("{}", x)),
			_ => None,
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockExtrinsic<T>> for BlockSignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: BlockExtrinsic<T>) -> Result<Self, Self::Error> {
		let Some(signature) = value.signature else {
			return Err("No signature found in extrinsic")?;
		};

		Ok(Self::new(signature, value.call, value.metadata))
	}
}

impl<T: HasHeader + Decode> TryFrom<BlockRawExtrinsic> for BlockSignedExtrinsic<T> {
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
	pub fn new(events: Vec<ExtrinsicEvent>) -> Self {
		Self { events }
	}

	pub fn first<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

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

	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::proxy::events::ProxyExecuted>()?;
		Some(executed.result.is_ok())
	}

	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::multisig::events::MultisigExecuted>()?;
		Some(executed.result.is_ok())
	}

	pub fn is_present<T: HasHeader>(&self) -> bool {
		self.count::<T>() > 0
	}

	pub fn is_present_parts(&self, pallet_id: u8, variant_id: u8) -> bool {
		self.count_parts(pallet_id, variant_id) > 0
	}

	pub fn count<T: HasHeader>(&self) -> u32 {
		self.count_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1)
	}

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
