use crate::subxt_signer::{SecretUri, sr25519::Keypair};
use avail_rust_core::{AccountId, H256, ext::subxt_core::utils::AccountId32};

#[cfg(feature = "generated_metadata")]
use crate::subxt_core::tx::payload::DefaultPayload;
#[cfg(feature = "generated_metadata")]
use crate::{Client, SubmittableTransaction};
#[cfg(feature = "generated_metadata")]
use avail_rust_core::TransactionCall;

pub trait H256Ext {
	fn from_str(s: &str) -> Result<H256, String>;
}

impl H256Ext for H256 {
	fn from_str(s: &str) -> Result<H256, String> {
		let mut s = s;
		if s.starts_with("0x") {
			s = &s[2..];
		}

		if s.len() != 64 {
			let msg = std::format!(
				"Failed to convert string to H256. Expected 64 bytes got {}. Input string: {}",
				s.len(),
				s
			);
			return Err(msg);
		}

		let block_hash = const_hex::decode(s).map_err(|e| e.to_string())?;
		let block_hash = TryInto::<[u8; 32]>::try_into(block_hash);
		match block_hash {
			Ok(v) => Ok(H256(v)),
			Err(e) => {
				let msg = std::format!("Failed to covert decoded string to H256. Input {:?}", e);
				Err(msg)
			},
		}
	}
}

pub trait AccountIdExt {
	fn from_str(value: &str) -> Result<AccountId, String>;
	fn from_slice(value: &[u8]) -> Result<AccountId, String>;
	fn default() -> AccountId;
}

impl AccountIdExt for AccountId {
	fn from_str(value: &str) -> Result<AccountId, String> {
		value.parse().map_err(|e| std::format!("{:?}", e))
	}

	fn from_slice(value: &[u8]) -> Result<AccountId, String> {
		let account_id: [u8; 32] = match value.try_into() {
			Ok(x) => x,
			Err(err) => return Err(err.to_string()),
		};

		Ok(AccountId { 0: account_id })
	}

	fn default() -> AccountId {
		AccountId32([0u8; 32])
	}
}

pub trait SecretUriExt {
	fn from_str(value: &str) -> Result<SecretUri, String>;
}

impl SecretUriExt for SecretUri {
	fn from_str(value: &str) -> Result<SecretUri, String> {
		value.parse().map_err(|e| std::format!("{:?}", e))
	}
}

pub trait KeypairExt {
	fn from_str(value: &str) -> Result<Keypair, String>;
	fn account_id(&self) -> AccountId;
}

impl KeypairExt for Keypair {
	fn from_str(value: &str) -> Result<Keypair, String> {
		let secret_uri = SecretUri::from_str(value).map_err(|e| e.to_string())?;
		let keypair = Keypair::from_uri(&secret_uri).map_err(|e| e.to_string())?;
		Ok(keypair)
	}

	fn account_id(&self) -> AccountId {
		self.public_key().to_account_id()
	}
}

#[cfg(feature = "generated_metadata")]
pub trait DefaultPayloadExt {
	fn to_transaction_call(&self, client: &Client) -> Result<TransactionCall, String>;
	fn to_submittable_transaction(&self, client: Client) -> Result<SubmittableTransaction, String>;
}

#[cfg(feature = "generated_metadata")]
impl<CallData: crate::codec::Encode> DefaultPayloadExt for DefaultPayload<CallData> {
	fn to_transaction_call(&self, client: &Client) -> Result<TransactionCall, String> {
		let pallet_name = self.pallet_name();
		let call_name = self.call_name();

		let metadata = client.online_client().metadata();
		let Some(pallet) = metadata.pallet_by_name(pallet_name) else {
			return Err("Failed to find pallet index".into());
		};
		let Some(call_variant) = pallet.call_variant_by_name(call_name) else {
			return Err("Failed to find call index".into());
		};

		let pallet_index = pallet.index();
		let call_index = call_variant.index;
		let call_data = self.call_data().encode();

		let value = TransactionCall::new(pallet_index, call_index, call_data);

		Ok(value)
	}

	fn to_submittable_transaction(&self, client: Client) -> Result<SubmittableTransaction, String> {
		let call = self.to_transaction_call(&client)?;
		let value = SubmittableTransaction::new(client, call);

		Ok(value)
	}
}
