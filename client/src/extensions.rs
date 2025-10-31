use crate::{
	UserError,
	subxt_signer::{SecretUri, sr25519::Keypair},
};
use avail_rust_core::{
	AccountId, H256,
	ext::subxt_core::utils::AccountId32,
	utils::{account_id_from_slice, account_id_from_str},
};

/// Extension helpers for working with `H256` values.
pub trait H256Ext {
	/// Parses a string (with or without `0x`) into an `H256`.
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

/// Extension helpers for constructing `AccountId` values.
pub trait AccountIdExt {
	/// Parses an address string into an `AccountId`.
	fn from_str(value: &str) -> Result<AccountId, String>;
	/// Decodes an `AccountId` from raw bytes.
	fn from_slice(value: &[u8]) -> Result<AccountId, String>;
	/// Returns the zero `AccountId`.
	fn default() -> AccountId;
}

impl AccountIdExt for AccountId {
	fn from_str(value: &str) -> Result<AccountId, String> {
		account_id_from_str(value)
	}

	fn from_slice(value: &[u8]) -> Result<AccountId, String> {
		account_id_from_slice(value)
	}

	fn default() -> AccountId {
		AccountId32([0u8; 32])
	}
}

/// Extension helpers for parsing signer URIs.
pub trait SecretUriExt {
	/// Parses a secret URI string into a signer `SecretUri`.
	fn from_str(value: &str) -> Result<SecretUri, UserError>;
}

impl SecretUriExt for SecretUri {
	fn from_str(value: &str) -> Result<SecretUri, UserError> {
		value.parse().map_err(|e| UserError::Other(std::format!("{:?}", e)))
	}
}

/// Extension helpers for building and inspecting sr25519 keypairs.
pub trait KeypairExt {
	/// Parses a secret URI string into a sr25519 keypair.
	fn from_str(value: &str) -> Result<Keypair, UserError>;
	/// Derives the associated `AccountId` from the public key.
	fn account_id(&self) -> AccountId;
}

impl KeypairExt for Keypair {
	fn from_str(value: &str) -> Result<Keypair, UserError> {
		let secret_uri = SecretUri::from_str(value).map_err(|e| UserError::Other(e.to_string()))?;
		let keypair = Keypair::from_uri(&secret_uri).map_err(|e| UserError::Other(e.to_string()))?;
		Ok(keypair)
	}

	fn account_id(&self) -> AccountId {
		self.public_key().to_account_id()
	}
}
