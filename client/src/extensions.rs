use avail_rust_core::{AccountId, H256, ext::subxt_core::utils::AccountId32, utils::account_id_from_slice};

/// Extension helpers for working with `H256` values.
pub trait H256Ext {
	/// Parses a string (with or without `0x`) into an `H256`.
	///
	/// Returns an error if the string is not 64 characters (after removing prefix) or contains invalid hex.
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
	/// Decodes an `AccountId` from raw bytes.
	///
	/// Returns an error if the byte slice is not exactly 32 bytes.
	fn from_slice(value: &[u8]) -> Result<AccountId, String>;

	/// Returns the zero `AccountId`.
	///
	/// Returns an `AccountId` with all bytes set to zero.
	fn default() -> AccountId;
}

impl AccountIdExt for AccountId {
	fn from_slice(value: &[u8]) -> Result<AccountId, String> {
		account_id_from_slice(value)
	}

	fn default() -> AccountId {
		AccountId32([0u8; 32])
	}
}
