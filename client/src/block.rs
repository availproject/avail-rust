use crate::{Client, H256Ext};
use avail_rust_core::{
	EncodeSelector, H256, HasHeader, TransactionEventDecodable, avail,
	rpc::{
		self,
		system::fetch_extrinsics::{ExtrinsicFilter, SignerPayload},
	},
	types::metadata::HashStringNumber,
};

pub struct Block {
	pub sxt: BSxt,
	pub ext: BExt,
	pub rxt: BRxt,
	pub event: BEvent,
}

impl Block {
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		fn inner(client: Client, block_id: HashStringNumber) -> Block {
			Block { sxt: todo!(), ext: todo!(), rxt: todo!(), event: todo!() }
		}

		inner(client, block_id.into())
	}
}

pub struct BRxt {
	client: Client,
	block_id: HashStringNumber,
}

impl BRxt {
	pub async fn get(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
		opts: BlockExtOptions2,
	) -> Result<Option<BlockRawExtrinsic>, avail_rust_core::Error> {
		let extrinsic_id: HashStringNumber = extrinsic_id.into();

		let filter = match extrinsic_id {
			HashStringNumber::Hash(x) => ExtrinsicFilter::TxHash(vec![x]),
			HashStringNumber::String(x) => ExtrinsicFilter::TxHash(vec![H256::from_str(&x)?]),
			HashStringNumber::Number(x) => ExtrinsicFilter::TxIndex(vec![x]),
		};
		let filter = Some(filter);
		self.first(BlockExtOptions2 { filter, ..opts }).await
	}

	pub async fn first(&self, opts: BlockExtOptions2) -> Result<Option<BlockRawExtrinsic>, avail_rust_core::Error> {
		let result = self.all(opts).await?;
		Ok(result.first().cloned())
	}

	pub async fn last(&self, opts: BlockExtOptions2) -> Result<Option<BlockRawExtrinsic>, avail_rust_core::Error> {
		let mut result = self.all(opts).await?;
		Ok(result.pop())
	}

	pub async fn all(&self, mut opts: BlockExtOptions2) -> Result<Vec<BlockRawExtrinsic>, avail_rust_core::Error> {
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
				let metadata =
					BlockExtrinsicBase::new(x.ext_hash, x.ext_index, x.pallet_id, x.variant_id, self.block_id.clone());
				BlockRawExtrinsic::new(x.data, metadata, x.signer_payload)
			})
			.collect();

		Ok(result)
	}

	pub async fn count(&self, mut opts: BlockExtOptions2) -> Result<usize, avail_rust_core::Error> {
		opts.encode_as = Some(EncodeSelector::None);

		let result = self.all(opts).await?;
		Ok(result.len())
	}

	pub async fn exists(&self, mut opts: BlockExtOptions2) -> Result<bool, avail_rust_core::Error> {
		opts.encode_as = Some(EncodeSelector::None);

		let result = self.first(opts).await?;
		Ok(result.is_some())
	}
}

pub struct BSxt {}
pub struct BExt {}
pub struct BEvent {}

#[derive(Debug, Default, Clone)]
pub struct BlockExtOptions2 {
	filter: Option<ExtrinsicFilter>,
	ss58_address: Option<String>,
	app_id: Option<u32>,
	nonce: Option<u32>,
	encode_as: Option<EncodeSelector>,
}

impl Into<rpc::system::fetch_extrinsics::Options> for BlockExtOptions2 {
	fn into(self) -> rpc::system::fetch_extrinsics::Options {
		todo!()
	}
}

#[derive(Debug, Clone)]
pub struct BlockExtrinsicBase {
	pub ext_hash: H256,
	pub ext_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub block_id: HashStringNumber,
}

impl BlockExtrinsicBase {
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, block_id: HashStringNumber) -> Self {
		Self { ext_hash, ext_index, pallet_id, variant_id, block_id }
	}
}

#[derive(Debug, Clone)]
pub struct BlockRawExtrinsic {
	pub data: Option<String>,
	pub metadata: BlockExtrinsicBase,
	pub signer_payload: Option<SignerPayload>,
}

impl BlockRawExtrinsic {
	pub fn new(data: Option<String>, metadata: BlockExtrinsicBase, signer_payload: Option<SignerPayload>) -> Self {
		Self { data, metadata, signer_payload }
	}

	pub async fn events() -> Result<ExtrinsicEvents, avail_rust_core::Error> {
		todo!()
	}
}

pub struct ExtrinsicEvent {
	pub index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub data: String,
}

pub struct ExtrinsicEvents {
	pub events: Vec<ExtrinsicEvent>,
}

impl ExtrinsicEvents {
	pub fn new(events: Vec<ExtrinsicEvent>) -> Self {
		Self { events }
	}

	pub fn find<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let Some(event) = event else {
			return None;
		};

		T::decode_event(&event.data)
	}

	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.find::<avail::proxy::events::ProxyExecuted>()?;
		return Some(executed.result.is_ok());
	}

	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.find::<avail::multisig::events::MultisigExecuted>()?;
		return Some(executed.result.is_ok());
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
