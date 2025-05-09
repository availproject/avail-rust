pub mod reqwest;
pub mod rpc;
pub mod runtime_api;

use crate::{
	config::*,
	error::{ClientError, RpcError},
	transaction::SubmittedTransaction,
	transaction_options::Options,
	transactions::Transactions,
	AvailHeader, BlockState,
};
use log::info;
use primitive_types::H256;
use rpc::ChainBlock;
use std::{fmt::Debug, sync::Arc};
use subxt_core::ext::scale_encode::EncodeAsFields;
use subxt_core::{blocks::StaticExtrinsic, tx::payload::DefaultPayload};
use subxt_rpcs::RpcClient;
use subxt_signer::sr25519::Keypair;

#[cfg(feature = "subxt_metadata")]
pub type AccountData = crate::avail::runtime_types::pallet_balances::types::AccountData<u128>;
#[cfg(feature = "subxt_metadata")]
use crate::avail::system::storage::types::account::Account as AccountInfo;

#[cfg(not(feature = "subxt"))]
use codec::Decode;

#[cfg(not(feature = "subxt"))]
use subxt_core::client::ClientState;

type SharedCache = Arc<std::sync::Mutex<Cache>>;

const MAX_CHAIN_BLOCKS: usize = 3;

#[derive(Clone)]
pub struct Client {
	#[cfg(feature = "subxt")]
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub cache: SharedCache,
}

impl Client {
	pub async fn new(endpoint: &str) -> Result<Client, ClientError> {
		let rpc_client = reqwest::ReqwestClient::new(endpoint);
		let rpc_client = RpcClient::new(rpc_client);

		Self::new_custom(rpc_client).await
	}

	pub fn tx(&self) -> Transactions {
		Transactions::new(self.clone())
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
	pub async fn block(&self, at: H256) -> Result<Option<ChainBlock>, RpcError> {
		self.rpc_chain_get_block(Some(at)).await
	}

	pub async fn best_block(&self) -> Result<ChainBlock, RpcError> {
		let block = self.block(self.best_block_hash().await?).await?;
		let Some(block) = block else {
			return Err("Best block not found.".into());
		};
		Ok(block)
	}

	pub async fn finalized_block(&self) -> Result<ChainBlock, RpcError> {
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
	pub async fn nonce(&self, address: &str) -> Result<u32, RpcError> {
		self.rpc_system_account_next_index(address).await
	}

	pub async fn nonce_state(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, RpcError> {
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
}

#[cfg(feature = "subxt")]
impl Client {
	pub async fn new_custom(rpc_client: RpcClient) -> Result<Client, ClientError> {
		// Cloning RpcClient is cheaper and doesn't create a new connection
		let online_client = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;

		Ok(Self {
			online_client,
			rpc_client,
			cache: SharedCache::default(),
		})
	}

	// Subxt
	pub fn subxt_blocks(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	pub fn subxt_storage(&self) -> AStorageClient {
		self.online_client.storage()
	}

	pub fn subxt_constants(&self) -> AConstantsClient {
		self.online_client.constants()
	}

	// Submission
	/// TODO
	pub async fn sign<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<Vec<u8>, RpcError>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let options = options.build(self, &account_id).await?;
		let params = options.clone().build().await;
		if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data")
		{
			let err = RpcError::TransactionNotAllowed("Transaction is not compatible with non-zero AppIds".into());
			return Err(err);
		}

		let mut tx_client = self.online_client.tx();
		let signed_call = tx_client.create_signed(payload, signer, params).await?;
		Ok(signed_call.into_encoded())
	}

	pub async fn sign_and_submit<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<SubmittedTransaction, RpcError>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let options = options.build(self, &account_id).await?;
		let params = options.clone().build().await;
		if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data")
		{
			let err = RpcError::TransactionNotAllowed("Transaction is not compatible with non-zero AppIds".into());
			return Err(err);
		}

		let mut tx_client = self.online_client.tx();
		let signed_call = tx_client.create_signed(payload, signer, params).await?;
		let tx_hash = self.rpc_author_submit_extrinsic(signed_call.encoded()).await?;
		info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_height, options.mortality.period, options.nonce, account_id);

		Ok(SubmittedTransaction::new(self.clone(), tx_hash, account_id, &options))
	}
}

#[cfg(all(feature = "subxt", feature = "subxt_metadata"))]
impl Client {
	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, RpcError> {
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		Ok(storage.fetch_or_default(&address).await?)
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.best_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		Ok(storage.fetch_or_default(&address).await?)
	}

	pub async fn finalized_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.finalized_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		Ok(storage.fetch_or_default(&address).await?)
	}
}

#[cfg(all(feature = "subxt", not(feature = "subxt_metadata")))]
impl Client {
	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, RpcError> {
		let storage = self.subxt_storage().at(at);
		let address = crate::transactions::system::account(account_id);
		Ok(storage.fetch_or_default(&address).await?)
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.best_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::transactions::system::account(account_id);
		Ok(storage.fetch_or_default(&address).await?)
	}

	pub async fn finalized_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.finalized_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::transactions::system::account(account_id);
		Ok(storage.fetch_or_default(&address).await?)
	}
}

#[cfg(all(not(feature = "subxt"), feature = "subxt_metadata"))]
impl Client {
	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, RpcError> {
		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let address = crate::avail::storage().system().account(account_id);
		Ok(crate::transactions::system::full(&metadata, &address, &self, at).await)
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.best_block_hash().await?;
		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let address = crate::avail::storage().system().account(account_id);
		Ok(crate::transactions::system::full(&metadata, &address, &self, at).await)
	}

	pub async fn finalized_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.finalized_block_hash().await?;
		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let address = crate::avail::storage().system().account(account_id);
		Ok(crate::transactions::system::full(&metadata, &address, &self, at).await)
	}
}

#[cfg(all(not(feature = "subxt"), not(feature = "subxt_metadata")))]
impl Client {
	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, RpcError> {
		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let address = crate::transactions::system::account(account_id);
		Ok(crate::transactions::system::full(&metadata, &address, &self, at).await)
	}

	pub async fn best_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.best_block_hash().await?;
		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let address = crate::transactions::system::account(account_id);
		Ok(crate::transactions::system::full(&metadata, &address, &self, at).await)
	}

	pub async fn finalized_block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let at = self.finalized_block_hash().await?;
		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let address = crate::transactions::system::account(account_id);
		Ok(crate::transactions::system::full(&metadata, &address, &self, at).await)
	}
}

#[cfg(not(feature = "subxt"))]
impl Client {
	pub async fn new_custom(rpc_client: RpcClient) -> Result<Client, ClientError> {
		Ok(Self {
			rpc_client,
			cache: SharedCache::default(),
		})
	}

	// Submission
	/// TODO
	pub async fn sign<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<Vec<u8>, RpcError>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let options = options.build(self, &account_id).await?;
		let params = options.clone().build().await;
		if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data")
		{
			let err = RpcError::TransactionNotAllowed("Transaction is not compatible with non-zero AppIds".into());
			return Err(err);
		}

		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let gh = self.block_hash(0).await.unwrap().unwrap();
		let rt = self.rpc_state_get_runtime_version(Some(gh)).await.unwrap();
		let cs = ClientState::<AvailConfig> {
			genesis_hash: gh,
			runtime_version: subxt_core::client::RuntimeVersion {
				spec_version: rt.spec_version,
				transaction_version: rt.transaction_version,
			},
			metadata,
		};

		let tx = subxt_core::tx::create_v4_signed(payload, &cs, params)?;
		let tx = tx.sign(signer);

		/* 		let mut tx_client = self.online_client.tx();
		let signed_call = tx_client.create_signed(payload, signer, params).await?; */
		Ok(tx.into_encoded())
	}

	pub async fn sign_and_submit<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<SubmittedTransaction, RpcError>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let options = options.build(self, &account_id).await?;
		let params = options.clone().build().await;
		if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data")
		{
			let err = RpcError::TransactionNotAllowed("Transaction is not compatible with non-zero AppIds".into());
			return Err(err);
		}

		let rpc_metadata = self.rpc_state_get_metadata(None).await.unwrap();
		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).unwrap();
		let metadata = subxt_core::Metadata::try_from(frame_metadata).unwrap();
		let gh = self.block_hash(0).await.unwrap().unwrap();
		let rt = self.rpc_state_get_runtime_version(Some(gh)).await.unwrap();
		let cs = ClientState::<AvailConfig> {
			genesis_hash: gh,
			runtime_version: subxt_core::client::RuntimeVersion {
				spec_version: rt.spec_version,
				transaction_version: rt.transaction_version,
			},
			metadata,
		};

		let tx = subxt_core::tx::create_v4_signed(payload, &cs, params)?;
		let tx = tx.sign(signer);

		/* 		let mut tx_client = self.online_client.tx();
		let signed_call = tx_client.create_signed(payload, signer, params).await?; */
		let tx_hash = self.rpc_author_submit_extrinsic(tx.encoded()).await?;
		info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_height, options.mortality.period, options.nonce, account_id);

		Ok(SubmittedTransaction::new(self.clone(), tx_hash, account_id, &options))
	}
}

#[derive(Clone, Default)]
pub struct CachedChainBlocks {
	blocks: [Option<(H256, Arc<ChainBlock>)>; MAX_CHAIN_BLOCKS],
	ptr: usize,
}

impl CachedChainBlocks {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn find(&self, block_hash: H256) -> Option<Arc<ChainBlock>> {
		for (hash, block) in self.blocks.iter().flatten() {
			if *hash == block_hash {
				return Some(block.clone());
			}
		}

		None
	}

	pub fn push(&mut self, value: (H256, Arc<ChainBlock>)) {
		self.blocks[self.ptr] = Some(value);
		self.ptr += 1;
		if self.ptr >= MAX_CHAIN_BLOCKS {
			self.ptr = 0;
		}
	}
}

#[derive(Default)]
pub struct Cache {
	pub chain_blocks_cache: CachedChainBlocks,
}

impl Debug for Cache {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Cache").field("last_fetched_block", &"").finish()
	}
}
