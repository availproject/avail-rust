use std::str::FromStr;

use avail_rust_core::{
	AccountIdLike,
	avail::{balances::types::AccountData, system::types::AccountInfo},
	subxt_signer::{SecretUri, bip39::Mnemonic, sr25519::Keypair},
	types::HashStringNumber,
};

use crate::{BlockQueryMode, Client, Error, UserError, error_ops};

pub struct Account<'a> {
	client: &'a Client,
}

impl<'a> Account<'a> {
	pub(crate) fn new(client: &'a Client) -> Self {
		Self { client }
	}

	pub fn new_from_str(value: &str) -> Result<Keypair, UserError> {
		let secret_uri = SecretUri::from_str(value).map_err(|e| {
			UserError::ValidationFailed(std::format!("[op:{}] {}", error_ops::ErrorOperation::KeypairParse, e))
		})?;
		let keypair = Keypair::from_uri(&secret_uri).map_err(|e| {
			UserError::ValidationFailed(std::format!(
				"[op:{}] Failed to derive keypair: {}",
				error_ops::ErrorOperation::KeypairParse,
				e
			))
		})?;
		Ok(keypair)
	}

	pub fn new_from_phrase(mnemonic: &Mnemonic, password: Option<&str>) -> Result<Keypair, Error> {
		let keypair = Keypair::from_phrase(&mnemonic, password).map_err(|e| {
			UserError::ValidationFailed(std::format!(
				"[op:{}] Failed to derive keypair: {}",
				error_ops::ErrorOperation::KeypairParse,
				e
			))
		})?;
		Ok(keypair)
	}

	pub fn new_from_uri(uri: &SecretUri) -> Result<Keypair, Error> {
		let keypair = Keypair::from_uri(uri).map_err(|e| {
			UserError::ValidationFailed(std::format!(
				"[op:{}] Failed to derive keypair: {}",
				error_ops::ErrorOperation::KeypairParse,
				e
			))
		})?;
		Ok(keypair)
	}

	pub async fn info(&self, account_id: impl Into<AccountIdLike>, mode: BlockQueryMode) -> Result<AccountInfo, Error> {
		match mode {
			BlockQueryMode::Finalized => self.client.finalized().account_info(account_id).await,
			BlockQueryMode::Best => self.client.best().account_info(account_id).await,
		}
	}

	pub async fn info_at(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<AccountInfo, Error> {
		self.client.chain().account_info(account_id, at).await
	}

	pub async fn nonce(&self, account_id: impl Into<AccountIdLike>, mode: BlockQueryMode) -> Result<u32, Error> {
		match mode {
			BlockQueryMode::Finalized => self.client.finalized().account_nonce(account_id).await,
			BlockQueryMode::Best => self.client.best().account_nonce(account_id).await,
		}
	}

	pub async fn nonce_at(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<u32, Error> {
		self.client.chain().block_nonce(account_id, at).await
	}

	pub async fn balance(
		&self,
		account_id: impl Into<AccountIdLike>,
		mode: BlockQueryMode,
	) -> Result<AccountData, Error> {
		match mode {
			BlockQueryMode::Finalized => self.client.finalized().account_balance(account_id).await,
			BlockQueryMode::Best => self.client.best().account_balance(account_id).await,
		}
	}

	pub async fn balance_at(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<AccountData, Error> {
		self.client.chain().account_balance(account_id, at).await
	}
}
