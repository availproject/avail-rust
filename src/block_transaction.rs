use crate::{
	block::{read_account_id, read_multi_address, EventRecords},
	primitives::block::extrinsics_params::CheckAppId,
	AExtrinsicDetails, AExtrinsicEvents, AExtrinsicSignedExtensions, AccountId, AvailConfig,
};
use codec::Decode;
use primitive_types::H256;
use subxt::{blocks::StaticExtrinsic, config::signed_extensions, utils::MultiAddress};

pub struct BlockTransactions {
	pub inner: Vec<BlockTransaction>,
}

impl BlockTransactions {
	pub fn iter(&self) -> BlockTransactionsIter {
		BlockTransactionsIter {
			inner: self,
			index: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.inner.len()
	}

	pub fn index(&self, index: usize) -> &BlockTransaction {
		&self.inner[index]
	}

	pub fn index_mut(&mut self, index: usize) -> &mut BlockTransaction {
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
		if self.inner.inner.len() == 0 {
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

	pub fn signer(&self) -> Option<String> {
		match self.account_id() {
			Some(x) => Some(std::format!("{}", x)),
			None => None,
		}
	}

	pub fn multi_address(&self) -> Option<MultiAddress<AccountId, u32>> {
		read_multi_address(&self.inner)
	}

	pub fn account_id(&self) -> Option<AccountId> {
		read_account_id(&self.inner)
	}

	pub fn tx_signer(&self) -> Option<String> {
		match self.account_id() {
			Some(x) => Some(std::format!("{}", x)),
			None => None,
		}
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

		return None;
	}

	pub fn nonce(&self) -> Option<u32> {
		let signed = self.signed()?;
		signed
			.find::<signed_extensions::CheckNonce>()
			.ok()?
			.map(|e| e as u32)
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

	pub async fn events(&self) -> Result<AExtrinsicEvents, subxt::Error> {
		self.inner.events().await
	}

	pub fn signer(&self) -> Option<String> {
		match self.account_id() {
			Some(x) => Some(std::format!("{:?}", x)),
			None => None,
		}
	}

	pub fn account_id(&self) -> Option<AccountId> {
		let bytes = self.inner.signature_bytes()?;

		let tx_signer: [u8; 32] = match bytes.try_into() {
			Ok(x) => x,
			Err(_) => return None,
		};

		Some(AccountId::from(tx_signer))
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
		signed
			.find::<signed_extensions::CheckMortality<AvailConfig>>()
			.ok()?
			.map(|e| e)
	}

	pub fn nonce(&self) -> Option<u32> {
		let signed = self.signed()?;
		signed
			.find::<signed_extensions::CheckNonce>()
			.ok()?
			.map(|e| e as u32)
	}
}
