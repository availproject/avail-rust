use crate::{
	primitives::block::extrinsics_params::CheckAppId, AExtrinsicDetails, AExtrinsicEvents,
	AExtrinsicSignedExtensions, AccountId, AvailConfig,
};
use primitive_types::H256;
use subxt::{blocks::StaticExtrinsic, config::signed_extensions};

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
