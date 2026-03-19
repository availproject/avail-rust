use crate::{Error, error_ops};
use std::str::FromStr;

/// Helpers that convert between RPC-compatible hash identifiers.
pub mod hash_string_number {
	use super::*;
	use crate::chain::Chain;
	use avail_rust_core::{H256, HashNumber, types::HashStringNumber};

	/// Resolves a [`HashStringNumber`] into the concrete block hash.
	///
	/// Propagates RPC errors returned by [`Chain::block_hash`], or decoding errors when parsing strings.
	pub async fn to_hash(c: &Chain, value: impl Into<HashStringNumber>) -> Result<H256, Error> {
		async fn inner(c: &Chain, value: HashNumber) -> Result<H256, Error> {
			match value {
				HashNumber::Hash(x) => Ok(x),
				HashNumber::Number(x) => {
					let hash = c.block_hash(Some(x)).await?;
					let Some(hash) = hash else {
						return Err(Error::not_found_with_op(
							error_ops::ErrorOperation::ConversionToHash,
							std::format!("No block hash found for block height: {}", x),
						));
					};
					Ok(hash)
				},
				HashNumber::HashAndNumber((h, ..)) => Ok(h),
			}
		}

		let value = HashNumber::try_from(value.into())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::ConversionToHash, e))?;
		inner(c, value).await
	}
}

/// Helpers that convert flexible hash strings into concrete hashes.
pub mod hash_string {
	use super::*;
	use avail_rust_core::{H256, types::HashString};

	/// Converts the provided hash representation into an [`H256`].
	///
	/// Returns an [`Error`] if the string cannot be decoded into a valid hash.
	pub fn to_hash(value: impl Into<HashString>) -> Result<H256, Error> {
		fn inner(value: HashString) -> Result<H256, Error> {
			let hash = match value {
				HashString::Hash(x) => x,
				HashString::String(x) => H256::from_str(&x).map_err(|e| {
					Error::validation_with_op(error_ops::ErrorOperation::ConversionToHash, e.to_string())
				})?,
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
	/// Returns an [`Error`] when the underlying conversion fails.
	pub fn to_account_id(value: impl Into<AccountIdLike>) -> Result<AccountId, Error> {
		fn inner(value: AccountIdLike) -> Result<AccountId, Error> {
			let account_id: AccountId = value.try_into().map_err(|e| {
				UserError::ValidationFailed(std::format!(
					"[op:{}] {}",
					error_ops::ErrorOperation::ConversionToAccountId,
					e
				))
			})?;
			Ok(account_id)
		}

		inner(value.into())
	}
}
