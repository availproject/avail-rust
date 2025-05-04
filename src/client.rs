use crate::{
	avail::{runtime_types::pallet_balances::types::AccountData, system::storage::types::account::Account},
	client_rpc::ChainBlock,
	error::ClientError,
	transaction::{BlockId, SubmittedTransaction},
	AConstantsClient, AOnlineClient, AStorageClient, AccountId, AccountIdExt, AvailHeader, BlockState, Options, H256,
};
use log::info;
use std::{fmt::Debug, sync::Arc};
use subxt::{backend::rpc::RpcClient, blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
use subxt_signer::sr25519::Keypair;

#[cfg(feature = "native")]
use crate::http;

#[cfg(feature = "native")]
pub async fn http_api(endpoint: &str) -> Result<Client, ClientError> {
	let rpc_client = http::HttpClient::new(endpoint).map_err(|e| e.to_string())?;
	let rpc_client = RpcClient::new(rpc_client);

	// Cloning RpcClient is cheaper and doesn't create a new WS connection
	let api = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;
	let client = Client::new(api, rpc_client);

	Ok(client)
}

type SharedCache = Arc<std::sync::Mutex<Cache>>;

const MAX_CHAIN_BLOCKS: usize = 3;
#[derive(Clone)]
pub struct CachedChainBlocks {
	blocks: [Option<(H256, Arc<ChainBlock>)>; MAX_CHAIN_BLOCKS],
	ptr: usize,
}

impl CachedChainBlocks {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn find(&self, block_hash: H256) -> Option<Arc<ChainBlock>> {
		for value in self.blocks.iter() {
			if let Some((hash, block)) = value {
				if *hash == block_hash {
					return Some(block.clone());
				}
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

impl Default for CachedChainBlocks {
	fn default() -> Self {
		Self {
			blocks: [None, None, None],
			ptr: 0,
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

#[derive(Debug, Clone)]
pub struct Client {
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub cache: SharedCache,
}

impl Client {
	pub fn new(online_client: AOnlineClient, rpc_client: RpcClient) -> Client {
		Self {
			online_client,
			rpc_client,
			cache: SharedCache::default(),
		}
	}

	// Header
	pub async fn header(&self, at: H256) -> Result<Option<AvailHeader>, subxt::Error> {
		self.rpc_chain_get_header(Some(at)).await
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, subxt::Error> {
		let header = self.header(self.best_block_hash().await?).await?;
		let Some(header) = header else {
			let err = std::format!("Best block header not found.");
			return Err(subxt::Error::Other(err));
		};
		Ok(header)
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, subxt::Error> {
		let header = self.header(self.finalized_block_hash().await?).await?;
		let Some(header) = header else {
			let err = std::format!("Finalized block header not found.");
			return Err(subxt::Error::Other(err));
		};
		Ok(header)
	}

	// (RPC) Block
	pub async fn block(&self, at: H256) -> Result<Option<ChainBlock>, subxt::Error> {
		self.rpc_chain_get_block(Some(at)).await
	}

	pub async fn best_block(&self) -> Result<ChainBlock, subxt::Error> {
		let block = self.block(self.best_block_hash().await?).await?;
		let Some(block) = block else {
			let err = std::format!("Best block not found.");
			return Err(subxt::Error::Other(err));
		};
		Ok(block)
	}

	pub async fn finalized_block(&self) -> Result<ChainBlock, subxt::Error> {
		let block = self.block(self.finalized_block_hash().await?).await?;
		let Some(block) = block else {
			let err = std::format!("Finalized block not found.");
			return Err(subxt::Error::Other(err));
		};
		Ok(block)
	}

	// Block Hash
	pub async fn block_hash(&self, block_height: u32) -> Result<Option<H256>, subxt::Error> {
		self.rpc_chain_get_block_hash(Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, subxt::Error> {
		let hash = self.rpc_chain_get_block_hash(None).await?;
		let Some(hash) = hash else {
			let err = std::format!("Best block hash not found.");
			let err = subxt::Error::Block(subxt::error::BlockError::NotFound(err));
			return Err(err);
		};
		Ok(hash)
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, subxt::Error> {
		self.rpc_chain_get_finalized_head().await
	}

	// Block Height
	pub async fn block_height(&self, block_hash: H256) -> Result<Option<u32>, subxt::Error> {
		let header = self.rpc_chain_get_header(Some(block_hash)).await?;
		Ok(header.and_then(|x| Some(x.number)))
	}

	pub async fn best_block_height(&self) -> Result<u32, subxt::Error> {
		let header = self.best_block_header().await?;
		Ok(header.number)
	}

	pub async fn finalized_block_height(&self) -> Result<u32, subxt::Error> {
		let header = self.finalized_block_header().await?;
		Ok(header.number)
	}

	// Block Id
	pub async fn best_block_id(&self) -> Result<BlockId, subxt::Error> {
		let hash = self.best_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			let err = std::format!("Best block header not found.");
			return Err(subxt::Error::Other(err));
		};
		Ok(BlockId::from((hash, height)))
	}

	pub async fn finalized_block_id(&self) -> Result<BlockId, subxt::Error> {
		let hash = self.finalized_block_hash().await?;
		let height = self.block_height(hash).await?;
		let Some(height) = height else {
			let err = std::format!("Finalized block header not found.");
			return Err(subxt::Error::Other(err));
		};
		Ok(BlockId::from((hash, height)))
	}

	// Nonce
	pub async fn nonce(&self, address: &str) -> Result<u32, subxt::Error> {
		let account = AccountId::from_str(address)?;
		self.rpc_system_account_next_index(account.to_string()).await
	}

	pub async fn nonce_state(&self, address: &str, block_hash: H256) -> Result<u32, subxt::Error> {
		let account = AccountId::from_str(address)?;
		let block = self.online_client.blocks().at(block_hash).await?;

		Ok(block.account_nonce(&account).await? as u32)
	}

	pub async fn best_block_nonce(&self, address: &str) -> Result<u32, subxt::Error> {
		self.nonce_state(address, self.best_block_hash().await?).await
	}

	pub async fn finalized_block_nonce(&self, address: &str) -> Result<u32, subxt::Error> {
		self.nonce_state(address, self.finalized_block_hash().await?).await
	}

	// Balance
	pub async fn balance(&self, account_id: AccountId, at: H256) -> Result<AccountData<u128>, subxt::Error> {
		Ok(self.account_info(account_id, at).await?.data)
	}

	pub async fn best_block_balance(&self, account_id: AccountId) -> Result<AccountData<u128>, subxt::Error> {
		Ok(self.best_block_account_info(account_id).await?.data)
	}

	pub async fn finalized_block_balance(&self, account_id: AccountId) -> Result<AccountData<u128>, subxt::Error> {
		Ok(self.finalized_block_account_info(account_id).await?.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: AccountId, at: H256) -> Result<Account, subxt::Error> {
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		storage.fetch_or_default(&address).await
	}

	pub async fn best_block_account_info(&self, account_id: AccountId) -> Result<Account, subxt::Error> {
		let at = self.best_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		storage.fetch_or_default(&address).await
	}

	pub async fn finalized_block_account_info(&self, account_id: AccountId) -> Result<Account, subxt::Error> {
		let at = self.finalized_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		storage.fetch_or_default(&address).await
	}

	// Block State
	pub async fn block_state(&self, block_id: BlockId) -> Result<BlockState, subxt::Error> {
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

	// Subxt
	pub fn subxt_storage(&self) -> AStorageClient {
		self.online_client.storage()
	}

	pub fn subxt_constants(&self) -> AConstantsClient {
		self.online_client.constants()
	}

	// Submission
	/// TODO
	pub async fn sign_and_submit<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<SubmittedTransaction, subxt::Error>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let options = options.build(self, &account_id).await?;
		let params = options.clone().build().await;
		if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data")
		{
			return Err(subxt::Error::Other(
				"Transaction is not compatible with non-zero AppIds".into(),
			));
		}

		let mut tx_client = self.online_client.tx();
		let signed_call = tx_client.create_signed(payload, signer, params).await?;
		let tx_hash = self.rpc_author_submit_extrinsic(signed_call.encoded()).await?;
		info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_height, options.mortality.period, options.nonce, account_id);

		Ok(SubmittedTransaction::new(self.clone(), tx_hash, account_id, &options))
	}
}
