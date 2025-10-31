use crate::{Error, H256Ext};

/// Helpers that convert between RPC-compatible hash identifiers.
pub mod hash_string_number {
	use super::*;
	use crate::chain::Chain;
	use avail_rust_core::{H256, HashNumber, types::HashStringNumber};

	/// Resolves a [`HashStringNumber`] into the concrete block hash.
	///
	/// # Arguments
	/// * `c` - Chain helper used to resolve block numbers to hashes when needed.
	/// * `value` - Hash, string, or height describing the target block.
	///
	/// # Returns
	/// Returns the resolved block hash (`H256`) or an error if the lookup fails.
	///
	/// # Errors
	/// Propagates RPC errors returned by [`Chain::block_hash`], or decoding errors when parsing strings.
	/// ```
	pub async fn to_hash(c: &Chain, value: impl Into<HashStringNumber>) -> Result<H256, Error> {
		async fn inner(c: &Chain, value: HashStringNumber) -> Result<H256, Error> {
			let hash = match value {
				HashStringNumber::Hash(x) => x,
				HashStringNumber::String(x) => H256::from_str(&x).map_err(Error::Other)?,
				HashStringNumber::Number(x) => {
					let hash = c.block_hash(Some(x)).await?;
					let Some(hash) = hash else {
						return Err(Error::Other(std::format!("No block hash was found for block height: {}", x)));
					};
					hash
				},
			};
			Ok(hash)
		}

		inner(c, value.into()).await
	}

	/// Converts a [`HashStringNumber`] into a [`HashNumber`].
	///
	/// # Arguments
	/// * `value` - Identifier representing either a block hash or height.
	///
	/// # Returns
	/// Returns a `HashNumber` preserving the original semantics.
	///
	/// # Errors
	/// Returns an [`Error`] when string decoding into a hash fails.
	pub fn to_hash_number(value: impl Into<HashStringNumber>) -> Result<HashNumber, Error> {
		fn inner(value: HashStringNumber) -> Result<HashNumber, Error> {
			let hash_number = match value {
				HashStringNumber::Hash(x) => HashNumber::Hash(x),
				HashStringNumber::String(x) => HashNumber::Hash(H256::from_str(&x).map_err(Error::Other)?),
				HashStringNumber::Number(x) => HashNumber::Number(x),
			};
			Ok(hash_number)
		}

		inner(value.into())
	}
}

/// Helpers that convert flexible hash strings into concrete hashes.
pub mod hash_string {
	use super::*;
	use avail_rust_core::{H256, types::HashString};

	/// Converts the provided hash representation into an [`H256`].
	///
	/// # Arguments
	/// * `value` - Hash or string encoding of the target hash.
	///
	/// # Returns
	/// Returns the concrete `H256` hash.
	///
	/// # Errors
	/// Returns an [`Error`] if the string cannot be decoded into a valid hash.
	pub fn to_hash(value: impl Into<HashString>) -> Result<H256, Error> {
		fn inner(value: HashString) -> Result<H256, Error> {
			let hash = match value {
				HashString::Hash(x) => x,
				HashString::String(x) => H256::from_str(&x).map_err(Error::Other)?,
			};
			Ok(hash)
		}

		inner(value.into())
	}
}

/// Helpers for converting account identifiers into canonical [`crate::AccountId`] values.
pub mod account_id_like {
	use super::*;
	use crate::UserError;
	use avail_rust_core::{AccountId, AccountIdLike};

	/// Converts an account identifier into the canonical [`crate::AccountId`].
	///
	/// # Arguments
	/// * `value` - Account representation accepted by RPC helpers.
	///
	/// # Returns
	/// Returns the resolved `AccountId`.
	///
	/// # Errors
	/// Returns an [`Error`] when the underlying conversion fails.
	pub fn to_account_id(value: impl Into<AccountIdLike>) -> Result<AccountId, Error> {
		fn inner(value: AccountIdLike) -> Result<AccountId, Error> {
			let account_id: AccountId = value.try_into().map_err(UserError::Other)?;
			Ok(account_id)
		}

		inner(value.into())
	}
}
