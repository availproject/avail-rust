use super::{
	block_client::BlockClient, cache_client::CacheClient, event_client::EventClient, online_client::OnlineClientT,
	storage_client::StorageClient,
};
use crate::{
	avail,
	subxt_rpcs::{
		methods::legacy::{RuntimeVersion, SystemHealth},
		RpcClient,
	},
	subxt_signer::sr25519::Keypair,
	transaction::SubmittedTransaction,
	transaction_options::Options,
	transactions::Transactions,
	BlockState,
};
use avail::{balances::types::AccountData, system::types::AccountInfo};
use client_core::{
	rpc,
	rpc::{
		kate::{BlockLength, Cell, GDataProof, GRow, ProofResponse},
		substrate::{
			BlockWithJustifications, NodeRole, PeerInfo, RpcMethods, SessionKeys, SyncState, SystemProperties,
		},
	},
	AccountId, AvailHeader, BlockId, H256,
};
use std::sync::Arc;

#[cfg(feature = "subxt")]
use crate::config::{ABlocksClient, AConstantsClient, AStorageClient};

#[derive(Clone)]
pub struct Client {
	#[cfg(not(feature = "subxt"))]
	online_client: super::online_client::OnlineClient,
	#[cfg(feature = "subxt")]
	online_client: crate::config::AOnlineClient,
	pub rpc_client: RpcClient,
	cache_client: CacheClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	pub async fn new(endpoint: &str) -> Result<Client, client_core::Error> {
		let rpc_client = super::reqwest_client::ReqwestClient::new(endpoint);
		let rpc_client = RpcClient::new(rpc_client);

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
	) -> Result<Client, client_core::Error> {
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
	) -> Result<Client, client_core::Error> {
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

	pub fn toggle_caching(&self, value: bool) {
		self.cache_client.toggle_caching(value);
	}

	// Header
	pub async fn header(&self, at: H256) -> Result<Option<AvailHeader>, client_core::Error> {
		self.rpc_chain_get_header(Some(at)).await
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, client_core::Error> {
		let header = self.header(self.best_block_hash().await?).await?;
		let Some(header) = header else {
			return Err("Best block header not found.".into());
		};
		Ok(header)
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, client_core::Error> {
		let header = self.header(self.finalized_block_hash().await?).await?;
		let Some(header) = header else {
			return Err("Finalized block header not found.".into());
		};
		Ok(header)
	}

	// (RPC) Block
	pub async fn block(&self, at: H256) -> Result<Option<BlockWithJustifications>, client_core::Error> {
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

	pub async fn best_block(&self) -> Result<BlockWithJustifications, client_core::Error> {
		let block = self.block(self.best_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Best block not found.".into());
		};
		Ok(block)
	}

	pub async fn finalized_block(&self) -> Result<BlockWithJustifications, client_core::Error> {
		let block = self.block(self.finalized_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Finalized block not found.".into());
		};
		Ok(block)
	}

	// Block Hash
	pub async fn block_hash(&self, block_height: u32) -> Result<Option<H256>, client_core::Error> {
		self.rpc_chain_get_block_hash(Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, client_core::Error> {
		let hash = self.rpc_chain_get_block_hash(None).await?;
		let Some(hash) = hash else {
			return Err("Best block hash not found.".into());
		};
		Ok(hash)
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, client_core::Error> {
		self.rpc_chain_get_finalized_head().await
	}

	// Block Height
	pub async fn block_height(&self, block_hash: H256) -> Result<Option<u32>, client_core::Error> {
		let header = self.rpc_chain_get_header(Some(block_hash)).await?;
		Ok(header.map(|x| x.number))
	}

	pub async fn best_block_height(&self) -> Result<u32, client_core::Error> {
		self.best_block_header().await.map(|x| x.number)
	}

	pub async fn finalized_block_height(&self) -> Result<u32, client_core::Error> {
		self.finalized_block_header().await.map(|x| x.number)
	}

	// Block Id
	pub async fn best_block_id(&self) -> Result<BlockId, client_core::Error> {
		let hash = self.best_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			return Err("Best block header not found.".into());
		};
		Ok(BlockId::from((hash, height)))
	}

	pub async fn finalized_block_id(&self) -> Result<BlockId, client_core::Error> {
		let hash = self.finalized_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			return Err("Finalized block header not found.".into());
		};
		Ok(BlockId::from((hash, height)))
	}

	// Nonce
	pub async fn nonce(&self, account_id: &AccountId) -> Result<u32, client_core::Error> {
		self.rpc_system_account_next_index(&std::format!("{}", account_id))
			.await
	}

	pub async fn block_nonce(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, client_core::Error> {
		self.account_info(account_id, block_hash).await.map(|x| x.nonce)
	}

	pub async fn best_block_nonce(&self, account_id: &AccountId) -> Result<u32, client_core::Error> {
		self.best_block_account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn finalized_block_nonce(&self, account_id: &AccountId) -> Result<u32, client_core::Error> {
		self.finalized_block_account_info(account_id).await.map(|v| v.nonce)
	}

	// Balance
	pub async fn balance(&self, account_id: &AccountId, at: H256) -> Result<AccountData, client_core::Error> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	pub async fn best_block_balance(&self, account_id: &AccountId) -> Result<AccountData, client_core::Error> {
		self.best_block_account_info(account_id).await.map(|x| x.data)
	}

	pub async fn finalized_block_balance(&self, account_id: &AccountId) -> Result<AccountData, client_core::Error> {
		self.finalized_block_account_info(account_id).await.map(|x| x.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, client_core::Error> {
		let address = avail::system::storage::account(account_id);
		let storage = self.storage_client();
		storage.fetch_or_default(&address, at).await
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, client_core::Error> {
		let at = self.best_block_hash().await?;
		let address = avail::system::storage::account(account_id);
		let storage = self.storage_client();
		storage.fetch_or_default(&address, at).await
	}

	pub async fn finalized_block_account_info(
		&self,
		account_id: &AccountId,
	) -> Result<AccountInfo, client_core::Error> {
		let at = self.finalized_block_hash().await?;
		let address = avail::system::storage::account(account_id);
		let storage = self.storage_client();
		storage.fetch_or_default(&address, at).await
	}

	// Block State
	pub async fn block_state(&self, block_id: BlockId) -> Result<BlockState, client_core::Error> {
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
	pub async fn sign_and_submit<'a>(&self, tx: &client_core::Transaction<'a>) -> Result<H256, client_core::Error> {
		let encoded = tx.encode();
		let tx_hash = self.rpc_author_submit_extrinsic(&encoded).await?;

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signed {
			if let client_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "lib", "Transaction submitted. Tx Hash: {:?}, Address: {}, Nonce: {}, App Id: {}", tx_hash, account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: client_core::TransactionPayload<'a>,
	) -> Result<H256, client_core::Error> {
		use client_core::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);
		let tx_hash = self.sign_and_submit(&tx).await?;

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &client_core::TransactionCall,
		options: Options,
	) -> Result<SubmittedTransaction, client_core::Error> {
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(self, &account_id).await?;

		let tx_extra = client_core::TransactionExtra::from(&refined_options);
		let tx_additional = client_core::TransactionAdditional {
			spec_version: self.online_client.spec_version(),
			tx_version: self.online_client.transaction_version(),
			genesis_hash: self.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = client_core::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
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
}

impl Client {
	pub async fn rpc_block_overview(
		&self,
		params: rpc::block::block_overview::Params,
	) -> Result<rpc::block::block_overview::Response, client_core::Error> {
		Ok(rpc::block::block_overview(&self.rpc_client, params).await?)
	}

	pub async fn rpc_block_data(
		&self,
		params: rpc::block::block_data::Params,
	) -> Result<rpc::block::block_data::Response, client_core::Error> {
		Ok(rpc::block::block_data(&self.rpc_client, params).await?)
	}

	pub async fn rpc_system_account_next_index(&self, address: &str) -> Result<u32, client_core::Error> {
		Ok(rpc::substrate::system_account_next_index(&self.rpc_client, address).await?)
	}

	pub async fn rpc_system_chain(&self) -> Result<String, client_core::Error> {
		Ok(rpc::substrate::system_chain(&self.rpc_client).await?)
	}

	pub async fn rpc_system_chain_type(&self) -> Result<String, client_core::Error> {
		Ok(rpc::substrate::system_chain_type(&self.rpc_client).await?)
	}

	pub async fn rpc_system_health(&self) -> Result<SystemHealth, client_core::Error> {
		Ok(rpc::substrate::system_health(&self.rpc_client).await?)
	}

	pub async fn rpc_system_local_listen_addresses(&self) -> Result<Vec<String>, client_core::Error> {
		Ok(rpc::substrate::system_local_listen_addresses(&self.rpc_client).await?)
	}

	pub async fn rpc_system_local_peer_id(&self) -> Result<String, client_core::Error> {
		Ok(rpc::substrate::system_local_peer_id(&self.rpc_client).await?)
	}

	pub async fn rpc_system_name(&self) -> Result<String, client_core::Error> {
		Ok(rpc::substrate::system_name(&self.rpc_client).await?)
	}

	pub async fn rpc_system_node_roles(&self) -> Result<Vec<NodeRole>, client_core::Error> {
		Ok(rpc::substrate::system_node_roles(&self.rpc_client).await?)
	}

	pub async fn rpc_system_peers(&self) -> Result<Vec<PeerInfo>, client_core::Error> {
		Ok(rpc::substrate::system_peers(&self.rpc_client).await?)
	}

	pub async fn rpc_system_properties(&self) -> Result<SystemProperties, client_core::Error> {
		Ok(rpc::substrate::system_properties(&self.rpc_client).await?)
	}

	pub async fn rpc_system_sync_state(&self) -> Result<SyncState, client_core::Error> {
		Ok(rpc::substrate::system_sync_state(&self.rpc_client).await?)
	}

	pub async fn rpc_system_version(&self) -> Result<String, client_core::Error> {
		Ok(rpc::substrate::system_version(&self.rpc_client).await?)
	}

	pub async fn rpc_chain_get_block(
		&self,
		at: Option<H256>,
	) -> Result<Option<BlockWithJustifications>, client_core::Error> {
		Ok(rpc::substrate::chain_get_block(&self.rpc_client, at).await?)
	}

	pub async fn rpc_chain_get_block_hash(
		&self,
		block_height: Option<u32>,
	) -> Result<Option<H256>, client_core::Error> {
		Ok(rpc::substrate::chain_get_block_hash(&self.rpc_client, block_height).await?)
	}

	pub async fn rpc_chain_get_finalized_head(&self) -> Result<H256, client_core::Error> {
		Ok(rpc::substrate::chain_get_finalized_head(&self.rpc_client).await?)
	}

	pub async fn rpc_chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, client_core::Error> {
		Ok(rpc::substrate::chain_get_header(&self.rpc_client, at).await?)
	}

	pub async fn rpc_author_rotate_keys(&self) -> Result<SessionKeys, client_core::Error> {
		Ok(rpc::substrate::author_rotate_keys(&self.rpc_client).await?)
	}

	pub async fn rpc_author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, client_core::Error> {
		Ok(rpc::substrate::author_submit_extrinsic(&self.rpc_client, extrinsic).await?)
	}

	pub async fn rpc_state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, client_core::Error> {
		Ok(rpc::substrate::state_get_runtime_version(&self.rpc_client, at).await?)
	}

	pub async fn rpc_state_call(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<String, client_core::Error> {
		Ok(rpc::substrate::state_call(&self.rpc_client, method, data, at).await?)
	}

	pub async fn rpc_state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, client_core::Error> {
		Ok(rpc::substrate::state_get_metadata(&self.rpc_client, at).await?)
	}

	pub async fn rpc_state_get_storage(
		&self,
		key: &str,
		at: Option<H256>,
	) -> Result<Option<Vec<u8>>, client_core::Error> {
		Ok(rpc::substrate::state_get_storage(&self.rpc_client, key, at).await?)
	}

	pub async fn rpc_rpc_methods(&self) -> Result<RpcMethods, client_core::Error> {
		Ok(rpc::substrate::rpc_methods(&self.rpc_client).await?)
	}

	pub async fn rpc_chainspec_v1_genesishash(&self) -> Result<H256, client_core::Error> {
		Ok(rpc::substrate::chainspec_v1_genesishash(&self.rpc_client).await?)
	}

	pub async fn rpc_kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, client_core::Error> {
		Ok(rpc::kate::kate_block_length(&self.rpc_client, at).await?)
	}

	pub async fn rpc_kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, client_core::Error> {
		Ok(rpc::kate::kate_query_data_proof(&self.rpc_client, transaction_index, at).await?)
	}

	pub async fn rpc_kate_query_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, client_core::Error> {
		Ok(rpc::kate::kate_query_proof(&self.rpc_client, cells, at).await?)
	}

	pub async fn rpc_kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, client_core::Error> {
		Ok(rpc::kate::kate_query_rows(&self.rpc_client, rows, at).await?)
	}
}
