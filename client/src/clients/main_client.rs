use super::{block_client::BlockClient, event_client::EventClient, online_client::OnlineClientT};
use crate::{
	BlockState, avail,
	clients::{rpc_api::RpcAPI, runtime_api::RuntimeApi},
	platform::sleep,
	subscription::{self, Subscriber},
	subxt_rpcs::RpcClient,
	subxt_signer::sr25519::Keypair,
	transaction::SubmittedTransaction,
	transaction_options::Options,
	transactions::Transactions,
};
use avail::{
	balances::types::AccountData,
	system::{storage as SystemStorage, types::AccountInfo},
};
use avail_rust_core::{AccountId, AvailHeader, BlockLocation, H256, StorageMap, rpc::BlockWithJustifications};
use std::time::Duration;

#[cfg(feature = "subxt")]
use crate::config::{ABlocksClient, AConstantsClient, AStorageClient};

#[derive(Clone)]
pub struct Client {
	#[cfg(not(feature = "subxt"))]
	online_client: super::online_client::OnlineClient,
	#[cfg(feature = "subxt")]
	online_client: crate::config::AOnlineClient,
	pub rpc_client: RpcClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	pub async fn new(endpoint: &str) -> Result<Client, avail_rust_core::Error> {
		let rpc_client = super::reqwest_client::ReqwestClient::new(endpoint);
		let rpc_client = RpcClient::new(rpc_client);

		Self::new_rpc_client(rpc_client).await
	}

	pub async fn new_rpc_client(rpc_client: RpcClient) -> Result<Client, avail_rust_core::Error> {
		#[cfg(not(feature = "subxt"))]
		let online_client = super::online_client::SimpleOnlineClient::new(&rpc_client).await?;
		#[cfg(feature = "subxt")]
		let online_client = crate::config::AOnlineClient::from_rpc_client(rpc_client.clone()).await?;

		Self::new_custom(rpc_client, online_client.into()).await
	}

	#[cfg(not(feature = "subxt"))]
	pub async fn new_custom(
		rpc_client: RpcClient,
		online_client: super::online_client::OnlineClient,
	) -> Result<Client, avail_rust_core::Error> {
		Ok(Self { online_client, rpc_client })
	}

	#[cfg(feature = "subxt")]
	pub async fn new_custom(
		rpc_client: RpcClient,
		online_client: crate::config::AOnlineClient,
	) -> Result<Client, avail_rust_core::Error> {
		Ok(Self { online_client, rpc_client })
	}

	pub fn tx(&self) -> Transactions {
		Transactions(self.clone())
	}

	#[cfg(feature = "tracing")]
	pub fn enable_tracing(enable_json_format: bool) {
		use tracing_subscriber::util::SubscriberInitExt;

		let builder = tracing_subscriber::fmt::SubscriberBuilder::default();
		if enable_json_format {
			let builder = builder.json();
			builder.finish().init();
		} else {
			builder.finish().init();
		}
	}

	// Header
	pub async fn block_header(&self, at: H256) -> Result<Option<AvailHeader>, avail_rust_core::Error> {
		self.rpc_api().chain_get_header(Some(at)).await
	}

	pub async fn block_header_with_retries(&self, at: H256) -> Result<Option<AvailHeader>, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];

		loop {
			let header = self.block_header(at).await;
			let header = match header {
				Ok(x) => x,
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err);
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching block header ended with Err {}. Sleep for {} seconds",
						err.to_string(),
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};

			if let Some(header) = header {
				return Ok(Some(header));
			}

			let Some(duration) = sleep_duration.pop() else {
				return Ok(None);
			};

			#[cfg(feature = "tracing")]
			trace_warn(&std::format!("Fetching block header ended with Option<None>. Sleep for {} seconds", duration));
			sleep(Duration::from_secs(duration)).await;
		}
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, avail_rust_core::Error> {
		let block_hash = self.best_block_hash().await?;
		let block_header = self.block_header_with_retries(block_hash).await?;
		let Some(block_header) = block_header else {
			return Err("Failed to find best block header.".into());
		};

		Ok(block_header)
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, avail_rust_core::Error> {
		let block_hash = self.finalized_block_hash().await?;
		let block_header = self.block_header_with_retries(block_hash).await?;
		let Some(block_header) = block_header else {
			return Err("Failed to find best block header.".into());
		};

		Ok(block_header)
	}

	// (RPC) Block
	pub async fn block(&self, at: H256) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		let block = self.rpc_api().chain_get_block(Some(at)).await?;
		if let Some(block) = block {
			Ok(Some(block))
		} else {
			Ok(None)
		}
	}

	pub async fn block_with_retries(
		&self,
		at: H256,
	) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			let block = self.rpc_api().chain_get_block(Some(at)).await;
			let block = match block {
				Ok(x) => x,
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err);
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching block ended with Err {}. Sleep for {} seconds",
						err.to_string(),
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};

			if let Some(block) = block {
				return Ok(Some(block));
			}

			let Some(duration) = sleep_duration.pop() else {
				return Ok(None);
			};

			#[cfg(feature = "tracing")]
			trace_warn(&std::format!("Fetching block ended with Option<None>. Sleep for {} seconds", duration));
			sleep(Duration::from_secs(duration)).await;
		}
	}

	pub async fn best_block(&self) -> Result<BlockWithJustifications, avail_rust_core::Error> {
		let block_hash = self.best_block_hash().await?;
		let block = self.block_with_retries(block_hash).await?;
		let Some(block) = block else {
			return Err("Best block not found.".into());
		};

		Ok(block)
	}

	pub async fn finalized_block(&self) -> Result<BlockWithJustifications, avail_rust_core::Error> {
		let block_hash = self.finalized_block_hash().await?;
		let block = self.block_with_retries(block_hash).await?;
		let Some(block) = block else {
			return Err("Finalized block not found.".into());
		};

		Ok(block)
	}

	// Block Hash
	pub async fn block_hash(&self, block_height: u32) -> Result<Option<H256>, avail_rust_core::Error> {
		self.rpc_api().chain_get_block_hash(Some(block_height)).await
	}

	pub async fn block_hash_with_retries(&self, block_height: u32) -> Result<Option<H256>, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			let hash = self.rpc_api().chain_get_block_hash(Some(block_height)).await;
			let hash = match hash {
				Ok(x) => x,
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err);
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching block hash ended with Err {}. Sleep for {} seconds",
						err.to_string(),
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};

			if let Some(hash) = hash {
				return Ok(Some(hash));
			}

			let Some(duration) = sleep_duration.pop() else {
				return Ok(None);
			};

			#[cfg(feature = "tracing")]
			trace_warn(&std::format!("Fetching block hash ended with Option<None>. Sleep for {} seconds", duration));
			sleep(Duration::from_secs(duration)).await;
		}
	}

	pub async fn best_block_hash(&self) -> Result<H256, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			let hash = self.rpc_api().chain_get_block_hash(None).await;
			let hash = match hash {
				Ok(x) => x,
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err);
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching best block hash ended with Err {}. Sleep for {} seconds",
						err.to_string(),
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};

			if let Some(hash) = hash {
				return Ok(hash);
			}

			let Some(duration) = sleep_duration.pop() else {
				return Err("Failed to fetch best block hash".into());
			};

			#[cfg(feature = "tracing")]
			trace_warn(&std::format!(
				"Fetching best block hash ended with Option<None>. Sleep for {} seconds",
				duration
			));
			sleep(Duration::from_secs(duration)).await;
		}
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			let hash = self.rpc_api().chain_get_finalized_head().await;
			match hash {
				Ok(x) => return Ok(x),
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err);
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching finalized block hash ended with Option<None>. Sleep for {} seconds",
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};
		}
	}

	// Block Height
	pub async fn block_height(&self, block_hash: H256) -> Result<Option<u32>, avail_rust_core::Error> {
		let header = self.block_header(block_hash).await?;
		Ok(header.map(|x| x.number))
	}

	pub async fn block_height_with_retries(&self, block_hash: H256) -> Result<Option<u32>, avail_rust_core::Error> {
		let header = self.block_header_with_retries(block_hash).await?;
		Ok(header.map(|x| x.number))
	}

	pub async fn best_block_height(&self) -> Result<u32, avail_rust_core::Error> {
		self.best_block_header().await.map(|x| x.number)
	}

	pub async fn finalized_block_height(&self) -> Result<u32, avail_rust_core::Error> {
		self.finalized_block_header().await.map(|x| x.number)
	}

	// Block Id
	pub async fn best_block_loc(&self) -> Result<BlockLocation, avail_rust_core::Error> {
		let hash = self.best_block_hash().await?;
		let height = self.block_height_with_retries(hash).await?;
		let Some(height) = height else {
			return Err("Best block header not found.".into());
		};
		Ok(BlockLocation::from((hash, height)))
	}

	pub async fn finalized_block_loc(&self) -> Result<BlockLocation, avail_rust_core::Error> {
		let hash = self.finalized_block_hash().await?;
		let height = self.block_height_with_retries(hash).await?;
		let Some(height) = height else {
			return Err("Finalized block header not found.".into());
		};
		Ok(BlockLocation::from((hash, height)))
	}

	// Nonce
	pub async fn nonce(&self, account_id: &AccountId) -> Result<u32, avail_rust_core::Error> {
		self.rpc_api()
			.system_account_next_index(&std::format!("{}", account_id))
			.await
	}

	pub async fn block_nonce(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, avail_rust_core::Error> {
		self.account_info(account_id, block_hash).await.map(|x| x.nonce)
	}

	pub async fn best_block_nonce(&self, account_id: &AccountId) -> Result<u32, avail_rust_core::Error> {
		self.best_block_account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn finalized_block_nonce(&self, account_id: &AccountId) -> Result<u32, avail_rust_core::Error> {
		self.finalized_block_account_info(account_id).await.map(|v| v.nonce)
	}

	// Balance
	pub async fn balance(&self, account_id: &AccountId, at: H256) -> Result<AccountData, avail_rust_core::Error> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	pub async fn best_block_balance(&self, account_id: &AccountId) -> Result<AccountData, avail_rust_core::Error> {
		self.best_block_account_info(account_id).await.map(|x| x.data)
	}

	pub async fn finalized_block_balance(&self, account_id: &AccountId) -> Result<AccountData, avail_rust_core::Error> {
		self.finalized_block_account_info(account_id).await.map(|x| x.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, avail_rust_core::Error> {
		SystemStorage::Account::fetch(&self.rpc_client, account_id, Some(at))
			.await
			.map(|x| x.unwrap_or_default())
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, avail_rust_core::Error> {
		let at = self.best_block_hash().await?;
		Self::account_info(&self, account_id, at).await
	}

	pub async fn finalized_block_account_info(
		&self,
		account_id: &AccountId,
	) -> Result<AccountInfo, avail_rust_core::Error> {
		let at = self.finalized_block_hash().await?;
		Self::account_info(&self, account_id, at).await
	}

	// Block State
	pub async fn block_state(&self, block_loc: BlockLocation) -> Result<BlockState, avail_rust_core::Error> {
		let real_block_hash = self.block_hash(block_loc.height).await?;
		let Some(real_block_hash) = real_block_hash else {
			return Ok(BlockState::DoesNotExist);
		};

		let finalized_block_height = self.finalized_block_height().await?;
		if block_loc.height > finalized_block_height {
			return Ok(BlockState::Included);
		}

		if block_loc.hash != real_block_hash {
			return Ok(BlockState::Discarded);
		}

		Ok(BlockState::Finalized)
	}

	// Sign and submit
	pub async fn submit<'a>(&self, tx: &avail_rust_core::Transaction<'a>) -> Result<H256, avail_rust_core::Error> {
		let encoded = tx.encode();
		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signed {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Submitting Transaction. Address: {}, Nonce: {}, App Id: {}", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}
		let tx_hash = self.rpc_api().author_submit_extrinsic(&encoded).await?;

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signed {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Transaction Submitted.  Address: {}, Nonce: {}, App Id: {}, Tx Hash: {:?},", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id, tx_hash);
			}
		}

		Ok(tx_hash)
	}

	pub async fn sign_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: avail_rust_core::TransactionPayload<'a>,
	) -> Result<avail_rust_core::Transaction<'a>, avail_rust_core::Error> {
		use avail_rust_core::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);

		Ok(tx)
	}

	pub async fn sign_call<'a>(
		&self,
		signer: &Keypair,
		tx_call: &'a avail_rust_core::TransactionCall,
		options: Options,
	) -> Result<avail_rust_core::Transaction<'a>, avail_rust_core::Error> {
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(self, &account_id).await?;

		let tx_extra = avail_rust_core::TransactionExtra::from(&refined_options);
		let tx_additional = avail_rust_core::TransactionAdditional {
			spec_version: self.online_client.spec_version(),
			tx_version: self.online_client.transaction_version(),
			genesis_hash: self.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		self.sign_payload(signer, tx_payload).await
	}

	pub async fn sign_and_submit_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: avail_rust_core::TransactionPayload<'a>,
	) -> Result<H256, avail_rust_core::Error> {
		use avail_rust_core::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);
		let tx_hash = self.submit(&tx).await?;

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &avail_rust_core::TransactionCall,
		options: Options,
	) -> Result<SubmittedTransaction, avail_rust_core::Error> {
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(self, &account_id).await?;

		let tx_extra = avail_rust_core::TransactionExtra::from(&refined_options);
		let tx_additional = avail_rust_core::TransactionAdditional {
			spec_version: self.online_client.spec_version(),
			tx_version: self.online_client.transaction_version(),
			genesis_hash: self.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		let tx_hash = self.sign_and_submit_payload(signer, tx_payload).await?;

		let value = SubmittedTransaction::new(self.clone(), tx_hash, account_id, refined_options, tx_additional);
		Ok(value)
	}

	// Mini Clients
	pub fn event_client(&self) -> EventClient {
		EventClient::new(self.clone())
	}

	pub fn block_client(&self) -> BlockClient {
		BlockClient::new(self.clone())
	}

	#[cfg(not(feature = "subxt"))]
	pub fn online_client(&self) -> super::online_client::OnlineClient {
		self.online_client.clone()
	}

	#[cfg(feature = "subxt")]
	pub fn online_client(&self) -> crate::config::AOnlineClient {
		self.online_client.clone()
	}

	// Subxt
	#[cfg(feature = "subxt")]
	pub fn subxt_blocks_client(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	#[cfg(feature = "subxt")]
	pub fn subxt_storage_client(&self) -> AStorageClient {
		self.online_client.storage()
	}

	#[cfg(feature = "subxt")]
	pub fn subxt_constants_client(&self) -> AConstantsClient {
		self.online_client.constants()
	}

	// Api
	pub fn runtime_api(&self) -> RuntimeApi {
		RuntimeApi::new(self.clone())
	}

	pub fn rpc_api(&self) -> RpcAPI {
		RpcAPI::new(self.clone())
	}

	// Subscription
	pub fn subscription_block_header(&self, sub: Subscriber) -> subscription::HeaderSubscription {
		subscription::HeaderSubscription::new(self.clone(), sub)
	}

	pub fn subscription_block(&self, sub: Subscriber) -> subscription::BlockSubscription {
		subscription::BlockSubscription::new(self.clone(), sub)
	}

	pub fn subscription_grandpa_justifications(
		&self,
		block_height: u32,
		poll_rate_ms: u64,
	) -> subscription::GrandpaJustificationsSubscription {
		subscription::GrandpaJustificationsSubscription::new(self.clone(), poll_rate_ms, block_height)
	}
}

#[cfg(feature = "tracing")]
fn trace_warn(message: &str) {
	tracing::warn!(target: "lib", message);
}
