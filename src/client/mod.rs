pub mod block_client;
pub mod cache_client;
pub mod event_client;
pub mod foreign;
pub mod online_client;
pub mod rpc;

#[cfg(feature = "reqwest")]
pub mod reqwest;

use std::sync::Arc;

use crate::{avail, primitives};
use crate::{
	error::RpcError, transaction::SubmittedTransaction, transaction_options::Options, transactions::Transactions,
	AvailHeader, BlockState,
};
use avail::balances::types::AccountData;
use avail::system::types::AccountInfo;
use block_client::BlockClient;
use cache_client::CacheClient;
use event_client::EventClient;
use log::info;
use online_client::OnlineClientT;
use primitive_types::H256;
use primitives::{rpc::substrate::SignedBlock, AccountId, BlockId};
use subxt_rpcs::RpcClient;
use subxt_signer::sr25519::Keypair;

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
	pub async fn new(endpoint: &str) -> Result<Client, RpcError> {
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
	) -> Result<Client, RpcError> {
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
	) -> Result<Client, RpcError> {
		Ok(Self {
			online_client,
			rpc_client,
			cache: SharedCache::default(),
		})
	}

	pub fn tx(&self) -> Transactions {
		Transactions(self.clone())
	}

	pub fn enable_logging() {
		env_logger::builder().init();
	}

	// Header
	pub async fn header(&self, at: H256) -> Result<Option<AvailHeader>, RpcError> {
		self.rpc_chain_get_header(Some(at)).await
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, RpcError> {
		let header = self.header(self.best_block_hash().await?).await?;
		let Some(header) = header else {
			return Err("Best block header not found.".into());
		};
		Ok(header)
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, RpcError> {
		let header = self.header(self.finalized_block_hash().await?).await?;
		let Some(header) = header else {
			return Err("Finalized block header not found.".into());
		};
		Ok(header)
	}

	// (RPC) Block
	pub async fn block(&self, at: H256) -> Result<Option<SignedBlock>, RpcError> {
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

	pub async fn best_block(&self) -> Result<SignedBlock, RpcError> {
		let block = self.block(self.best_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Best block not found.".into());
		};
		Ok(block)
	}

	pub async fn finalized_block(&self) -> Result<SignedBlock, RpcError> {
		let block = self.block(self.finalized_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Finalized block not found.".into());
		};
		Ok(block)
	}

	// Block Hash
	pub async fn block_hash(&self, block_height: u32) -> Result<Option<H256>, RpcError> {
		self.rpc_chain_get_block_hash(Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, RpcError> {
		let hash = self.rpc_chain_get_block_hash(None).await?;
		let Some(hash) = hash else {
			return Err("Best block hash not found.".into());
		};
		Ok(hash)
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, RpcError> {
		self.rpc_chain_get_finalized_head().await
	}

	// Block Height
	pub async fn block_height(&self, block_hash: H256) -> Result<Option<u32>, RpcError> {
		let header = self.rpc_chain_get_header(Some(block_hash)).await?;
		Ok(header.map(|x| x.number))
	}

	pub async fn best_block_height(&self) -> Result<u32, RpcError> {
		self.best_block_header().await.map(|x| x.number)
	}

	pub async fn finalized_block_height(&self) -> Result<u32, RpcError> {
		self.finalized_block_header().await.map(|x| x.number)
	}

	// Block Id
	pub async fn best_block_id(&self) -> Result<BlockId, RpcError> {
		let hash = self.best_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			return Err("Best block header not found.".into());
		};
		Ok(BlockId::from((hash, height)))
	}

	pub async fn finalized_block_id(&self) -> Result<BlockId, RpcError> {
		let hash = self.finalized_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			return Err("Finalized block header not found.".into());
		};
		Ok(BlockId::from((hash, height)))
	}

	// Nonce
	pub async fn nonce(&self, account_id: &AccountId) -> Result<u32, RpcError> {
		self.rpc_system_account_next_index(&std::format!("{}", account_id))
			.await
	}

	pub async fn block_nonce(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, RpcError> {
		self.account_info(account_id, block_hash).await.map(|x| x.nonce)
	}

	pub async fn best_block_nonce(&self, account_id: &AccountId) -> Result<u32, RpcError> {
		self.best_block_account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn finalized_block_nonce(&self, account_id: &AccountId) -> Result<u32, RpcError> {
		self.finalized_block_account_info(account_id).await.map(|v| v.nonce)
	}

	// Balance
	pub async fn balance(&self, account_id: &AccountId, at: H256) -> Result<AccountData, RpcError> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	pub async fn best_block_balance(&self, account_id: &AccountId) -> Result<AccountData, RpcError> {
		self.best_block_account_info(account_id).await.map(|x| x.data)
	}

	pub async fn finalized_block_balance(&self, account_id: &AccountId) -> Result<AccountData, RpcError> {
		self.finalized_block_account_info(account_id).await.map(|x| x.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, RpcError> {
		let address = avail::system::storage::account(account_id);
		self.storage_fetch_or_default(&address, at).await
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.best_block_hash().await?;
		let address = avail::system::storage::account(account_id);
		self.storage_fetch_or_default(&address, at).await
	}

	pub async fn finalized_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.finalized_block_hash().await?;
		let address = avail::system::storage::account(account_id);
		self.storage_fetch_or_default(&address, at).await
	}

	// Block State
	pub async fn block_state(&self, block_id: BlockId) -> Result<BlockState, RpcError> {
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
	pub async fn sign_and_submit<'a>(&self, tx: &primitives::Transaction<'a>) -> Result<H256, RpcError> {
		let encoded = tx.encode();
		let tx_hash = self.rpc_author_submit_extrinsic(&encoded).await?;

		if let Some(signed) = &tx.signed {
			if let primitives::MultiAddress::Id(account_id) = &signed.address {
				info!(target: "lib", "Transaction submitted. Tx Hash: {:?}, Address: {}, Nonce: {}, App Id: {}", tx_hash, account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: primitives::TransactionPayload<'a>,
	) -> Result<H256, RpcError> {
		use primitives::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);
		let tx_hash = self.sign_and_submit(&tx).await?;

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &primitives::TransactionCall,
		options: Options,
	) -> Result<SubmittedTransaction, RpcError> {
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(self, &account_id).await?;

		let tx_extra = primitives::TransactionExtra::from(&refined_options);
		let tx_additional = primitives::TransactionAdditional {
			spec_version: self.online_client.spec_version(),
			tx_version: self.online_client.transaction_version(),
			genesis_hash: self.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = primitives::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
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

#[cfg(feature = "subxt")]
impl Client {
	// Storage
	pub async fn storage_fetch<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Option<Addr::Target>, RpcError>
	where
		Addr: subxt_core::storage::address::Address<IsFetchable = subxt_core::utils::Yes> + 'address,
	{
		let storage = self.subxt_storage().at(at);
		return Ok(storage.fetch(address).await?);
	}

	pub async fn storage_fetch_or_default<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Addr::Target, RpcError>
	where
		Addr: subxt_core::storage::address::Address<
				IsFetchable = subxt_core::utils::Yes,
				IsDefaultable = subxt_core::utils::Yes,
			> + 'address,
	{
		let storage = self.subxt_storage().at(at);
		return Ok(storage.fetch_or_default(address).await?);
	}

	// constants
	pub async fn constants_at<'address, Addr>(&self, address: &Addr) -> Result<Addr::Target, RpcError>
	where
		Addr: subxt_core::constants::address::Address,
	{
		let val = self.subxt_constants().at(address)?;
		Ok(val)
	}
}

#[cfg(not(feature = "subxt"))]
impl Client {
	// Storage
	pub async fn storage_fetch<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Option<Addr::Target>, RpcError>
	where
		Addr: subxt_core::storage::address::Address<IsFetchable = subxt_core::utils::Yes> + 'address,
	{
		let metadata = self.online_client.metadata();
		let key = subxt_core::storage::get_address_bytes(address, &metadata)?;
		let key = std::format!("0x{}", hex::encode(key));
		if let Some(data) = self.rpc_state_get_storage(&key, Some(at)).await? {
			let val = subxt_core::storage::decode_value(&mut &*data, address, &metadata)?;
			Ok(Some(val))
		} else {
			Ok(None)
		}
	}

	pub async fn storage_fetch_or_default<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Addr::Target, RpcError>
	where
		Addr: subxt_core::storage::address::Address<
				IsFetchable = subxt_core::utils::Yes,
				IsDefaultable = subxt_core::utils::Yes,
			> + 'address,
	{
		if let Some(data) = self.storage_fetch(address, at).await? {
			Ok(data)
		} else {
			let metadata = self.online_client.metadata();
			let val = subxt_core::storage::default_value(address, &metadata)?;
			Ok(val)
		}
	}

	// constants
	pub async fn constants_at<'address, Addr>(&self, address: &Addr) -> Result<Addr::Target, RpcError>
	where
		Addr: subxt_core::constants::address::Address,
	{
		let metadata = self.online_client.metadata();
		let val = subxt_core::constants::get(address, &metadata)?;
		Ok(val)
	}
}
