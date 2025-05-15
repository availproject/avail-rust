use crate::primitives::AccountId;
use primitive_types::H256;
use subxt_signer::{sr25519::Keypair, SecretUri};

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

		let block_hash = hex::decode(s).map_err(|e| e.to_string())?;
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
