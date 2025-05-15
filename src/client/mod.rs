pub mod foreign;
pub mod reqwest;
pub mod rpc;

use crate::avail;
use crate::event_client::EventClient;
use crate::primitives;
use crate::primitives::rpc::substrate::SignedBlock;
use crate::primitives::AccountId;
use crate::primitives::BlockId;
use crate::{
	error::RpcError, transaction::SubmittedTransaction, transaction_options::Options, transactions::Transactions,
	AvailHeader, BlockState,
};
use avail::balances::types::AccountData;
use avail::system::types::AccountInfo;
use log::info;
use primitive_types::H256;
use std::{fmt::Debug, sync::Arc};
use subxt_rpcs::RpcClient;
use subxt_signer::sr25519::Keypair;

#[cfg(feature = "subxt")]
use crate::config::{ABlocksClient, AConstantsClient, AOnlineClient, AStorageClient};
#[cfg(not(feature = "subxt"))]
use codec::Decode;

type SharedCache = Arc<std::sync::Mutex<Cache>>;

const MAX_CHAIN_BLOCKS: usize = 3;

#[cfg(not(feature = "subxt"))]
#[derive(Clone)]
pub struct ClientState {
	pub genesis_hash: H256,
	pub spec_version: u32,
	pub tx_version: u32,
	pub metadata: subxt_core::Metadata,
}

#[derive(Clone)]
pub struct Client {
	#[cfg(feature = "subxt")]
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub cache: SharedCache,
	#[cfg(not(feature = "subxt"))]
	pub state: ClientState,
}

impl Client {
	pub async fn new(endpoint: &str) -> Result<Client, RpcError> {
		let rpc_client = reqwest::ReqwestClient::new(endpoint);
		let rpc_client = RpcClient::new(rpc_client);

		Self::new_custom(rpc_client).await
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
		self.rpc_chain_get_block(Some(at)).await
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
			spec_version: self.spec_version(),
			tx_version: self.transaction_version(),
			genesis_hash: self.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = primitives::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		let tx_hash = self.sign_and_submit_payload(signer, tx_payload).await?;

		let value = SubmittedTransaction::new(self.clone(), tx_hash, account_id, refined_options, tx_additional);
		Ok(value)
	}

	pub fn events_client(&self) -> EventClient {
		EventClient::new(self.clone())
	}
}

#[cfg(feature = "subxt")]
impl Client {
	pub async fn new_custom(rpc_client: RpcClient) -> Result<Client, RpcError> {
		// Cloning RpcClient is cheaper and doesn't create a new connection
		let online_client = AOnlineClient::from_rpc_client(rpc_client.clone())
			.await
			.map_err(|e| e.to_string())?;

		Ok(Self {
			online_client,
			rpc_client,
			cache: SharedCache::default(),
		})
	}

	pub fn metadata(&self) -> subxt_core::Metadata {
		self.online_client.metadata()
	}

	pub fn genesis_hash(&self) -> H256 {
		self.online_client.genesis_hash()
	}

	pub fn spec_version(&self) -> u32 {
		self.online_client.runtime_version().spec_version
	}

	pub fn transaction_version(&self) -> u32 {
		self.online_client.runtime_version().transaction_version
	}

	pub fn subxt_blocks(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	pub fn subxt_storage(&self) -> AStorageClient {
		self.online_client.storage()
	}

	pub fn subxt_constants(&self) -> AConstantsClient {
		self.online_client.constants()
	}

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
	pub async fn new_custom(rpc_client: RpcClient) -> Result<Client, RpcError> {
		use crate::primitives::rpc::substrate;
		let finalized_hash = substrate::chain_get_finalized_head(&rpc_client).await?;
		let rpc_metadata = substrate::state_get_metadata(&rpc_client, Some(finalized_hash)).await?;
		let genesis_hash = substrate::chainspec_v1_genesishash(&rpc_client).await?;
		let runtime_version = substrate::state_get_runtime_version(&rpc_client, Some(finalized_hash)).await?;

		let frame_metadata =
			frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).map_err(|e| e.to_string())?;
		let metadata = subxt_core::Metadata::try_from(frame_metadata).map_err(|e| e.to_string())?;

		let state = ClientState {
			genesis_hash,
			spec_version: runtime_version.spec_version,
			tx_version: runtime_version.transaction_version,
			metadata,
		};

		Ok(Self {
			rpc_client,
			cache: SharedCache::default(),
			state,
		})
	}

	pub fn metadata(&self) -> subxt_core::Metadata {
		self.state.metadata.clone()
	}

	pub fn genesis_hash(&self) -> H256 {
		self.state.genesis_hash
	}

	pub fn spec_version(&self) -> u32 {
		self.state.spec_version
	}

	pub fn transaction_version(&self) -> u32 {
		self.state.tx_version
	}

	// Storage
	pub async fn storage_fetch<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Option<Addr::Target>, RpcError>
	where
		Addr: subxt_core::storage::address::Address<IsFetchable = subxt_core::utils::Yes> + 'address,
	{
		let metadata = self.metadata();
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
			let metadata = self.metadata();
			let val = subxt_core::storage::default_value(address, &metadata)?;
			Ok(val)
		}
	}

	// constants
	pub async fn constants_at<'address, Addr>(&self, address: &Addr) -> Result<Addr::Target, RpcError>
	where
		Addr: subxt_core::constants::address::Address,
	{
		let metadata = self.metadata();
		let val = subxt_core::constants::get(address, &metadata)?;
		Ok(val)
	}
}

#[derive(Clone, Default)]
pub struct CachedChainBlocks {
	blocks: [Option<(H256, Arc<SignedBlock>)>; MAX_CHAIN_BLOCKS],
	ptr: usize,
}

impl CachedChainBlocks {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn find(&self, block_hash: H256) -> Option<Arc<SignedBlock>> {
		for (hash, block) in self.blocks.iter().flatten() {
			if *hash == block_hash {
				return Some(block.clone());
			}
		}

		None
	}

	pub fn push(&mut self, value: (H256, Arc<SignedBlock>)) {
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
