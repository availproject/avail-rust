use crate::{
	block::{read_account_id, read_multi_address, EventRecords},
	primitives::block::extrinsics_params::CheckAppId,
	AExtrinsicDetails, AExtrinsicSignedExtensions, AccountId,
};
use codec::Decode;
use primitive_types::H256;
use subxt::{blocks::StaticExtrinsic, config::signed_extensions, utils::MultiAddress};

#[derive(Debug, Clone, Default)]
pub struct Filter {
	pub app_id: Option<u32>,
	pub tx_hash: Option<H256>,
	pub tx_index: Option<u32>,
	pub tx_signer: Option<AccountId>,
}

impl Filter {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	pub fn tx_hash(mut self, value: H256) -> Self {
		self.tx_hash = Some(value);
		self
	}

	pub fn tx_index(mut self, value: u32) -> Self {
		self.tx_index = Some(value);
		self
	}

	pub fn tx_signer(mut self, value: AccountId) -> Self {
		self.tx_signer = Some(value);
		self
	}
}
pub struct BlockTransactions {
	pub inner: Vec<BlockTransaction>,
}

impl BlockTransactions {
	pub fn iter(&self) -> BlockTransactionsIter {
		BlockTransactionsIter { inner: self, index: 0 }
	}

	pub fn len(&self) -> usize {
		self.inner.len()
	}

	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}

	pub fn get(&self, index: usize) -> &BlockTransaction {
		&self.inner[index]
	}

	pub fn get_mut(&mut self, index: usize) -> &mut BlockTransaction {
		&mut self.inner[index]
	}
}

impl IntoIterator for BlockTransactions {
	type Item = BlockTransaction;
	type IntoIter = BlockTransactionsIntoIter;

	fn into_iter(self) -> Self::IntoIter {
		BlockTransactionsIntoIter { inner: self }
	}
}

pub struct BlockTransactionsIter<'a> {
	pub inner: &'a BlockTransactions,
	index: usize,
}

impl<'a> Iterator for BlockTransactionsIter<'a> {
	type Item = &'a BlockTransaction;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.inner.inner.len() {
			let result = Some(&self.inner.inner[self.index]);
			self.index += 1;
			result
		} else {
			None
		}
	}
}

pub struct BlockTransactionsIntoIter {
	pub inner: BlockTransactions,
}

impl Iterator for BlockTransactionsIntoIter {
	type Item = BlockTransaction;

	fn next(&mut self) -> Option<Self::Item> {
		if self.inner.inner.is_empty() {
			return None;
		}
		let result = self.inner.inner.remove(0);
		Some(result)
	}
}

pub struct BlockTransaction {
	pub inner: AExtrinsicDetails,
}

impl BlockTransaction {
	pub fn pallet_name(&self) -> Result<&str, subxt::Error> {
		self.inner.pallet_name()
	}

	pub fn pallet_index(&self) -> u8 {
		self.inner.pallet_index()
	}

	pub fn call_name(&self) -> Result<&str, subxt::Error> {
		self.inner.variant_name()
	}

	pub fn call_index(&self) -> u8 {
		self.inner.variant_index()
	}

	pub fn tx_hash(&self) -> H256 {
		self.inner.hash()
	}

	pub fn tx_index(&self) -> u32 {
		self.inner.index()
	}

	pub fn signed(&self) -> Option<AExtrinsicSignedExtensions> {
		self.inner.signed_extensions()
	}

	pub async fn events(&self) -> Option<EventRecords> {
		let ext_events = self.inner.events().await.ok()?;
		EventRecords::new_ext(ext_events)
	}

	pub fn multi_address(&self) -> Option<MultiAddress<AccountId, u32>> {
		read_multi_address(&self.inner)
	}

	pub fn account_id(&self) -> Option<AccountId> {
		read_account_id(&self.inner)
	}

	pub fn ss58address(&self) -> Option<String> {
		self.account_id().map(|x| std::format!("{}", x))
	}

	pub fn app_id(&self) -> Option<u32> {
		let signed = self.signed()?;

		signed.find::<CheckAppId>().ok()?.map(|e| e.0)
	}

	pub fn tip(&self) -> Option<u128> {
		let signed = self.signed()?;
		signed
			.find::<signed_extensions::ChargeTransactionPayment>()
			.ok()?
			.map(|e| e.tip())
	}

	pub fn mortality(&self) -> Option<subxt_core::utils::Era> {
		let signed = self.signed()?;
		for si in signed.iter() {
			if si.name() == "CheckMortality" {
				let mut bytes = si.bytes();
				let era = subxt_core::utils::Era::decode(&mut bytes).ok()?;
				return Some(era);
			}
		}

		None
	}

	pub fn nonce(&self) -> Option<u32> {
		let signed = self.signed()?;
		signed.find::<signed_extensions::CheckNonce>().ok()?.map(|e| e as u32)
	}

	pub fn decode<E: StaticExtrinsic>(&self) -> Option<E> {
		self.inner.as_extrinsic::<E>().ok().flatten()
	}
}

pub struct StaticBlockTransaction<E: StaticExtrinsic> {
	pub inner: AExtrinsicDetails,
	pub value: E,
}
impl<E: StaticExtrinsic> StaticBlockTransaction<E> {
	pub fn pallet_name(&self) -> Result<&str, subxt::Error> {
		self.inner.pallet_name()
	}

	pub fn pallet_index(&self) -> u8 {
		self.inner.pallet_index()
	}

	pub fn call_name(&self) -> Result<&str, subxt::Error> {
		self.inner.variant_name()
	}

	pub fn call_index(&self) -> u8 {
		self.inner.variant_index()
	}

	pub fn tx_hash(&self) -> H256 {
		self.inner.hash()
	}

	pub fn tx_index(&self) -> u32 {
		self.inner.index()
	}

	pub fn signed(&self) -> Option<AExtrinsicSignedExtensions> {
		self.inner.signed_extensions()
	}

	pub async fn events(&self) -> Option<EventRecords> {
		let ext_events = self.inner.events().await.ok()?;
		EventRecords::new_ext(ext_events)
	}

	pub fn multi_address(&self) -> Option<MultiAddress<AccountId, u32>> {
		read_multi_address(&self.inner)
	}

	pub fn account_id(&self) -> Option<AccountId> {
		read_account_id(&self.inner)
	}

	pub fn ss58address(&self) -> Option<String> {
		self.account_id().map(|x| std::format!("{}", x))
	}

	pub fn app_id(&self) -> Option<u32> {
		let signed = self.signed()?;

		signed.find::<CheckAppId>().ok()?.map(|e| e.0)
	}

	pub fn tip(&self) -> Option<u128> {
		let signed = self.signed()?;
		signed
			.find::<signed_extensions::ChargeTransactionPayment>()
			.ok()?
			.map(|e| e.tip())
	}

	pub fn mortality(&self) -> Option<subxt_core::utils::Era> {
		let signed = self.signed()?;
		for si in signed.iter() {
			if si.name() == "CheckMortality" {
				let mut bytes = si.bytes();
				let era = subxt_core::utils::Era::decode(&mut bytes).ok()?;
				return Some(era);
			}
		}

		None
	}

	pub fn nonce(&self) -> Option<u32> {
		let signed = self.signed()?;
		signed.find::<signed_extensions::CheckNonce>().ok()?.map(|e| e as u32)
	}
}
