pub mod block_client;
pub mod cache_client;
pub mod event_client;
pub mod online_client;
pub mod rpc;
pub mod storage_client;

#[cfg(feature = "reqwest")]
pub mod reqwest;

use std::sync::Arc;

use crate::{
	avail, subxt_rpcs::RpcClient, subxt_signer::sr25519::Keypair, transaction::SubmittedTransaction,
	transaction_options::Options, transactions::Transactions, BlockState,
};
use avail::{balances::types::AccountData, system::types::AccountInfo};
use block_client::BlockClient;
use cache_client::CacheClient;
use core::{rpc::substrate::BlockWithJustifications, AccountId, AvailHeader, BlockId, H256};
use event_client::EventClient;
use online_client::OnlineClientT;
use storage_client::StorageClient;

#[cfg(feature = "subxt")]
use crate::config::{ABlocksClient, AConstantsClient, AStorageClient};

#[derive(Clone)]
pub struct Client {
	#[cfg(not(feature = "subxt"))]
	online_client: online_client::OnlineClient,
	#[cfg(feature = "subxt")]
	online_client: crate::config::AOnlineClient,
	pub rpc_client: RpcClient,
	cache_client: CacheClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	pub async fn new(endpoint: &str) -> Result<Client, core::Error> {
		let rpc_client = reqwest::ReqwestClient::new(endpoint);
		let rpc_client = RpcClient::new(rpc_client);

		#[cfg(not(feature = "subxt"))]
		let online_client = online_client::SimpleOnlineClient::new(&rpc_client).await?;
		#[cfg(feature = "subxt")]
		let online_client = crate::config::AOnlineClient::from_rpc_client(rpc_client.clone()).await?;

		Self::new_custom(rpc_client, online_client.into()).await
	}

	#[cfg(not(feature = "subxt"))]
	pub async fn new_custom(
		rpc_client: RpcClient,
		online_client: online_client::OnlineClient,
	) -> Result<Client, core::Error> {
		Ok(Self {
			online_client,
			rpc_client,
			cache_client: CacheClient::new(),
		})
	}

	#[cfg(feature = "subxt")]
	pub async fn new_custom(
		rpc_client: RpcClient,
		online_client: crate::config::AOnlineClient,
	) -> Result<Client, core::Error> {
		Ok(Self {
			online_client,
			rpc_client,
			cache_client: CacheClient::new(),
		})
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
	pub async fn header(&self, at: H256) -> Result<Option<AvailHeader>, core::Error> {
		self.rpc_chain_get_header(Some(at)).await
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, core::Error> {
		let header = self.header(self.best_block_hash().await?).await?;
		let Some(header) = header else {
			return Err("Best block header not found.".into());
		};
		Ok(header)
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, core::Error> {
		let header = self.header(self.finalized_block_hash().await?).await?;
		let Some(header) = header else {
			return Err("Finalized block header not found.".into());
		};
		Ok(header)
	}

	// (RPC) Block
	pub async fn block(&self, at: H256) -> Result<Option<BlockWithJustifications>, core::Error> {
		if let Some(block) = self.cache_client.find_signed_block(at) {
			return Ok(Some(block.as_ref().clone()));
		}

		let block = self.rpc_chain_get_block(Some(at)).await?;
		if let Some(block) = block {
			self.cache_client.push_signed_block((at, Arc::new(block.clone())));
			Ok(Some(block))
		} else {
			Ok(None)
		}
	}

	pub async fn best_block(&self) -> Result<BlockWithJustifications, core::Error> {
		let block = self.block(self.best_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Best block not found.".into());
		};
		Ok(block)
	}

	pub async fn finalized_block(&self) -> Result<BlockWithJustifications, core::Error> {
		let block = self.block(self.finalized_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Finalized block not found.".into());
		};
		Ok(block)
	}

	// Block Hash
	pub async fn block_hash(&self, block_height: u32) -> Result<Option<H256>, core::Error> {
		self.rpc_chain_get_block_hash(Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, core::Error> {
		let hash = self.rpc_chain_get_block_hash(None).await?;
		let Some(hash) = hash else {
			return Err("Best block hash not found.".into());
		};
		Ok(hash)
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, core::Error> {
		self.rpc_chain_get_finalized_head().await
	}

	// Block Height
	pub async fn block_height(&self, block_hash: H256) -> Result<Option<u32>, core::Error> {
		let header = self.rpc_chain_get_header(Some(block_hash)).await?;
		Ok(header.map(|x| x.number))
	}

	pub async fn best_block_height(&self) -> Result<u32, core::Error> {
		self.best_block_header().await.map(|x| x.number)
	}

	pub async fn finalized_block_height(&self) -> Result<u32, core::Error> {
		self.finalized_block_header().await.map(|x| x.number)
	}

	// Block Id
	pub async fn best_block_id(&self) -> Result<BlockId, core::Error> {
		let hash = self.best_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			return Err("Best block header not found.".into());
		};
		Ok(BlockId::from((hash, height)))
	}

	pub async fn finalized_block_id(&self) -> Result<BlockId, core::Error> {
		let hash = self.finalized_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			return Err("Finalized block header not found.".into());
		};
		Ok(BlockId::from((hash, height)))
	}

	// Nonce
	pub async fn nonce(&self, account_id: &AccountId) -> Result<u32, core::Error> {
		self.rpc_system_account_next_index(&std::format!("{}", account_id))
			.await
	}

	pub async fn block_nonce(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, core::Error> {
		self.account_info(account_id, block_hash).await.map(|x| x.nonce)
	}

	pub async fn best_block_nonce(&self, account_id: &AccountId) -> Result<u32, core::Error> {
		self.best_block_account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn finalized_block_nonce(&self, account_id: &AccountId) -> Result<u32, core::Error> {
		self.finalized_block_account_info(account_id).await.map(|v| v.nonce)
	}

	// Balance
	pub async fn balance(&self, account_id: &AccountId, at: H256) -> Result<AccountData, core::Error> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	pub async fn best_block_balance(&self, account_id: &AccountId) -> Result<AccountData, core::Error> {
		self.best_block_account_info(account_id).await.map(|x| x.data)
	}

	pub async fn finalized_block_balance(&self, account_id: &AccountId) -> Result<AccountData, core::Error> {
		self.finalized_block_account_info(account_id).await.map(|x| x.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, core::Error> {
		let address = avail::system::storage::account(account_id);
		let storage = self.storage_client();
		storage.fetch_or_default(&address, at).await
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, core::Error> {
		let at = self.best_block_hash().await?;
		let address = avail::system::storage::account(account_id);
		let storage = self.storage_client();
		storage.fetch_or_default(&address, at).await
	}

	pub async fn finalized_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, core::Error> {
		let at = self.finalized_block_hash().await?;
		let address = avail::system::storage::account(account_id);
		let storage = self.storage_client();
		storage.fetch_or_default(&address, at).await
	}

	// Block State
	pub async fn block_state(&self, block_id: BlockId) -> Result<BlockState, core::Error> {
		let real_block_hash = self.block_hash(block_id.height).await?;
		let Some(real_block_hash) = real_block_hash else {
			return Ok(BlockState::DoesNotExist);
		};

		let finalized_block_height = self.finalized_block_height().await?;
		if block_id.height > finalized_block_height {
			return Ok(BlockState::Included);
		}

		if block_id.hash != real_block_hash {
			return Ok(BlockState::Discarded);
		}

		Ok(BlockState::Finalized)
	}

	// Sign and submit
	pub async fn sign_and_submit<'a>(&self, tx: &core::Transaction<'a>) -> Result<H256, core::Error> {
		let encoded = tx.encode();
		let tx_hash = self.rpc_author_submit_extrinsic(&encoded).await?;

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signed {
			if let core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "lib", "Transaction submitted. Tx Hash: {:?}, Address: {}, Nonce: {}, App Id: {}", tx_hash, account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: core::TransactionPayload<'a>,
	) -> Result<H256, core::Error> {
		use core::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);
		let tx_hash = self.sign_and_submit(&tx).await?;

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &core::TransactionCall,
		options: Options,
	) -> Result<SubmittedTransaction, core::Error> {
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(self, &account_id).await?;

		let tx_extra = core::TransactionExtra::from(&refined_options);
		let tx_additional = core::TransactionAdditional {
			spec_version: self.online_client.spec_version(),
			tx_version: self.online_client.transaction_version(),
			genesis_hash: self.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = core::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
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

	pub fn cache_client(&self) -> CacheClient {
		self.cache_client.clone()
	}

	pub fn storage_client(&self) -> StorageClient {
		StorageClient::new(self.clone())
	}

	#[cfg(not(feature = "subxt"))]
	pub fn online_client(&self) -> online_client::OnlineClient {
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
}
