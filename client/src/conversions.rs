use crate::{Error, H256Ext};

pub mod hash_string_number {
	use super::*;
	use crate::chain::Chain;
	use avail_rust_core::{H256, HashNumber, types::HashStringNumber};

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

pub mod hash_string {
	use super::*;
	use avail_rust_core::{H256, types::HashString};

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

pub mod account_id_like {
	use super::*;
	use crate::UserError;
	use avail_rust_core::{AccountId, AccountIdLike};

	pub fn to_account_id(value: impl Into<AccountIdLike>) -> Result<AccountId, Error> {
		fn inner(value: AccountIdLike) -> Result<AccountId, Error> {
			let account_id: AccountId = value.try_into().map_err(UserError::Other)?;
			Ok(account_id)
		}

		inner(value.into())
	}
}
