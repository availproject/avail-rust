use crate::{
	avail::data_availability::calls::types as DataAvailabilityCalls,
	block_transaction::StaticBlockTransaction, error::ClientError,
	primitives::block::extrinsics_params::CheckAppId, rpc, ABlock, AExtrinsicDetails,
	AExtrinsicEvents, AExtrinsics, AFoundExtrinsic, BlockTransaction, Client,
};
use primitive_types::H256;
use subxt::{
	backend::StreamOfResults, blocks::StaticExtrinsic, storage::StorageKeyValuePair, utils::Yes,
};

pub struct Block {
	pub block: ABlock,
	pub transactions: AExtrinsics,
}

impl Block {
	pub async fn new(client: &Client, block_hash: H256) -> Result<Self, ClientError> {
		let (block, transactions) = fetch_transactions(client, block_hash).await?;
		Ok(Self {
			block,
			transactions,
		})
	}

	pub async fn new_best_block(client: &Client) -> Result<Self, ClientError> {
		let block_hash = Self::fetch_best_block_hash(client).await?;
		Self::new(client, block_hash).await
	}

	pub async fn new_finalized_block(client: &Client) -> Result<Self, ClientError> {
		let block_hash = Self::fetch_finalized_block_hash(client).await?;
		Self::new(client, block_hash).await
	}

	pub async fn from_block(block: ABlock) -> Result<Self, subxt::Error> {
		let transactions = block.extrinsics().await?;
		Ok(Self {
			block,
			transactions,
		})
	}

	pub async fn from_block_number(
		client: &Client,
		block_number: u32,
	) -> Result<Self, ClientError> {
		let block_hash = rpc::chain::get_block_hash(client, Some(block_number)).await?;
		Self::new(client, block_hash).await
	}

	pub async fn events(
		&self,
		tx_index: Option<u32>,
	) -> Result<Vec<AExtrinsicEvents>, ClientError> {
		fetch_events(&self.transactions, tx_index).await
	}

	pub fn transaction_count(&self) -> usize {
		transaction_count(&self.transactions)
	}

	pub fn transaction_all_static<E: StaticExtrinsic>(&self) -> Vec<StaticBlockTransaction<E>> {
		transaction_all_static::<E>(&self.transactions)
	}

	pub fn transaction_by_signer_static<E: StaticExtrinsic>(
		&self,
		signer: &str,
	) -> Vec<StaticBlockTransaction<E>> {
		transaction_by_signer_static::<E>(&self.transactions, signer)
	}

	pub fn transaction_by_index_static<E: StaticExtrinsic>(
		&self,
		tx_index: u32,
	) -> Option<StaticBlockTransaction<E>> {
		transaction_by_index_static::<E>(&self.transactions, tx_index)
	}

	pub fn transaction_by_hash_static<E: StaticExtrinsic>(
		&self,
		tx_hash: H256,
	) -> Option<StaticBlockTransaction<E>> {
		transaction_by_hash_static::<E>(&self.transactions, tx_hash)
	}

	pub fn transaction_by_app_id_static<E: StaticExtrinsic>(
		&self,
		app_id: u32,
	) -> Vec<StaticBlockTransaction<E>> {
		transaction_by_app_id_static::<E>(&self.transactions, app_id)
	}

	pub fn transaction_by_hash(&self, tx_hash: H256) -> Option<BlockTransaction> {
		transaction_by_hash(&self.transactions, tx_hash)
	}

	pub fn transaction_by_signer(&self, signer: &str) -> Vec<BlockTransaction> {
		transaction_by_signer(&self.transactions, signer)
	}

	pub fn transaction_by_index(&self, tx_index: u32) -> Option<BlockTransaction> {
		transaction_by_index(&self.transactions, tx_index)
	}

	pub fn transaction_by_app_id(&self, app_id: u32) -> Vec<BlockTransaction> {
		transaction_by_app_id(&self.transactions, app_id)
	}

	pub fn data_submissions_all(&self) -> Vec<DataSubmission> {
		data_submissions_all(&self.transactions)
	}

	pub fn data_submissions_by_signer(&self, signer: &str) -> Vec<DataSubmission> {
		data_submissions_by_signer(&self.transactions, signer)
	}

	pub fn data_submissions_by_index(&self, tx_index: u32) -> Option<DataSubmission> {
		data_submissions_by_index(&self.transactions, tx_index)
	}

	pub fn data_submissions_by_hash(&self, tx_hash: H256) -> Option<DataSubmission> {
		data_submissions_by_hash(&self.transactions, tx_hash)
	}

	pub fn data_submissions_by_app_id(&self, app_id: u32) -> Vec<DataSubmission> {
		data_submissions_by_app_id(&self.transactions, app_id)
	}

	pub async fn storage_fetch<'address, T>(
		&self,
		address: &'address T,
	) -> Result<Option<<T as subxt::storage::Address>::Target>, subxt::Error>
	where
		T: subxt::storage::Address<IsFetchable = Yes> + 'address,
	{
		self.block.storage().fetch(address).await
	}

	pub async fn storage_fetch_or_default<'address, T>(
		&self,
		address: &'address T,
	) -> Result<<T as subxt::storage::Address>::Target, subxt::Error>
	where
		T: subxt::storage::Address<IsFetchable = Yes, IsDefaultable = Yes> + 'address,
	{
		self.block.storage().fetch_or_default(address).await
	}

	pub async fn storage_iter<T>(
		&self,
		address: T,
	) -> Result<StreamOfResults<StorageKeyValuePair<T>>, subxt::Error>
	where
		T: subxt::storage::Address<IsIterable = Yes> + 'static,
		T::Keys: 'static + Sized,
	{
		self.block.storage().iter(address).await
	}

	pub async fn fetch_best_block_hash(client: &Client) -> Result<H256, ClientError> {
		rpc::chain::get_block_hash(client, None).await
	}

	pub async fn fetch_finalized_block_hash(client: &Client) -> Result<H256, ClientError> {
		rpc::chain::get_finalized_head(client).await
	}

	pub fn transaction_hash_to_index(&self, tx_hash: H256) -> Option<u32> {
		transaction_hash_to_index(&self.transactions, tx_hash)
			.first()
			.cloned()
	}
}

pub async fn fetch_transactions(
	client: &Client,
	block_hash: H256,
) -> Result<(ABlock, AExtrinsics), subxt::Error> {
	let block = client.blocks().at(block_hash).await?;
	let transactions = block.extrinsics().await?;
	Ok((block, transactions))
}

pub async fn fetch_events(
	transactions: &AExtrinsics,
	tx_index: Option<u32>,
) -> Result<Vec<AExtrinsicEvents>, ClientError> {
	let mut events = Vec::new();
	let iter = transactions.iter();
	for details in iter {
		let ev = details.events().await?;
		if let Some(tx_index) = tx_index {
			if details.index() == tx_index {
				events.push(ev);
			}
		} else {
			events.push(ev);
		}
	}

	Ok(events)
}

pub fn transaction_count(transactions: &AExtrinsics) -> usize {
	transactions.len()
}

pub fn transaction_all_static<E: StaticExtrinsic>(
	transactions: &AExtrinsics,
) -> Vec<StaticBlockTransaction<E>> {
	transactions
		.iter()
		.flat_map(|details| match details.as_extrinsic::<E>().ok().flatten() {
			Some(x) => Some(StaticBlockTransaction {
				inner: details,
				value: x,
			}),
			None => None,
		})
		.collect()
}

pub fn transaction_by_signer_static<E: StaticExtrinsic>(
	transactions: &AExtrinsics,
	signer: &str,
) -> Vec<StaticBlockTransaction<E>> {
	transactions
		.iter()
		.flat_map(|details| {
			if details.signature_bytes() != Some(signer.as_bytes()) {
				return None;
			}

			return match details.as_extrinsic::<E>().ok().flatten() {
				Some(x) => Some(StaticBlockTransaction {
					inner: details,
					value: x,
				}),
				None => None,
			};
		})
		.collect()
}

pub fn transaction_by_index_static<E: StaticExtrinsic>(
	transactions: &AExtrinsics,
	tx_index: u32,
) -> Option<StaticBlockTransaction<E>> {
	transactions
		.iter()
		.flat_map(|details| {
			if details.index() != tx_index {
				return None;
			}

			return match details.as_extrinsic::<E>().ok().flatten() {
				Some(x) => Some(StaticBlockTransaction {
					inner: details,
					value: x,
				}),
				None => None,
			};
		})
		.next()
}

pub fn transaction_by_hash_static<E: StaticExtrinsic>(
	transactions: &AExtrinsics,
	tx_hash: H256,
) -> Option<StaticBlockTransaction<E>> {
	transactions
		.iter()
		.flat_map(|details| {
			if details.hash() != tx_hash {
				return None;
			}

			return match details.as_extrinsic::<E>().ok().flatten() {
				Some(x) => Some(StaticBlockTransaction {
					inner: details,
					value: x,
				}),
				None => None,
			};
		})
		.next()
}

pub fn transaction_by_app_id_static<E: StaticExtrinsic>(
	transactions: &AExtrinsics,
	app_id: u32,
) -> Vec<StaticBlockTransaction<E>> {
	transactions
		.iter()
		.flat_map(|details| {
			if read_app_id(&details) != Some(app_id) {
				return None;
			}

			return match details.as_extrinsic::<E>().ok().flatten() {
				Some(x) => Some(StaticBlockTransaction {
					inner: details,
					value: x,
				}),
				None => None,
			};
		})
		.collect()
}

pub fn transaction_by_signer(transactions: &AExtrinsics, signer: &str) -> Vec<BlockTransaction> {
	transactions
		.iter()
		.filter(|tx| tx.signature_bytes() == Some(signer.as_bytes()))
		.map(|tx| BlockTransaction { inner: tx })
		.collect()
}

pub fn transaction_by_index(transactions: &AExtrinsics, tx_index: u32) -> Option<BlockTransaction> {
	transactions
		.iter()
		.find(|tx| tx.index() == tx_index)
		.map(|tx| BlockTransaction { inner: tx })
}

pub fn transaction_by_hash(transactions: &AExtrinsics, tx_hash: H256) -> Option<BlockTransaction> {
	transactions
		.iter()
		.filter(|tx| tx.hash() == tx_hash)
		.next()
		.map(|tx| BlockTransaction { inner: tx })
}

pub fn transaction_by_app_id(transactions: &AExtrinsics, app_id: u32) -> Vec<BlockTransaction> {
	transactions
		.iter()
		.filter(|tx| read_app_id(tx) == Some(app_id))
		.map(|tx| BlockTransaction { inner: tx })
		.collect()
}

pub fn data_submissions_by_signer(transactions: &AExtrinsics, signer: &str) -> Vec<DataSubmission> {
	transaction_by_signer_static::<DataAvailabilityCalls::SubmitData>(transactions, signer)
		.into_iter()
		.map(DataSubmission::from_static_block_transaction)
		.collect()
}

pub fn data_submissions_all(transactions: &AExtrinsics) -> Vec<DataSubmission> {
	transaction_all_static::<DataAvailabilityCalls::SubmitData>(transactions)
		.into_iter()
		.map(DataSubmission::from_static_block_transaction)
		.collect()
}

pub fn data_submissions_by_index(
	transactions: &AExtrinsics,
	tx_index: u32,
) -> Option<DataSubmission> {
	transaction_by_index_static::<DataAvailabilityCalls::SubmitData>(transactions, tx_index)
		.map(DataSubmission::from_static_block_transaction)
}

pub fn data_submissions_by_hash(
	transactions: &AExtrinsics,
	tx_hash: H256,
) -> Option<DataSubmission> {
	transaction_by_hash_static::<DataAvailabilityCalls::SubmitData>(transactions, tx_hash)
		.map(DataSubmission::from_static_block_transaction)
}

pub fn data_submissions_by_app_id(transactions: &AExtrinsics, app_id: u32) -> Vec<DataSubmission> {
	transaction_by_app_id_static::<DataAvailabilityCalls::SubmitData>(transactions, app_id)
		.into_iter()
		.map(DataSubmission::from_static_block_transaction)
		.collect()
}

pub fn read_app_id(transaction: &AExtrinsicDetails) -> Option<u32> {
	transaction
		.signed_extensions()?
		.find::<CheckAppId>()
		.ok()?
		.map(|e| e.0)
}

pub fn transaction_hash_to_index(transactions: &AExtrinsics, tx_hash: H256) -> Vec<u32> {
	let mut indices = Vec::new();
	for tx in transactions.iter() {
		if tx.hash() == tx_hash {
			indices.push(tx.index())
		}
	}

	indices
}

#[derive(Debug, Clone)]
pub struct DataSubmission {
	pub tx_hash: H256,
	pub tx_index: u32,
	pub data: Vec<u8>,
	pub tx_signer: Vec<u8>,
	pub app_id: u32,
}

impl DataSubmission {
	pub fn from_static(tx: AFoundExtrinsic<DataAvailabilityCalls::SubmitData>) -> Self {
		let tx_hash = tx.details.hash();
		let tx_index = tx.details.index();
		let tx_signer = tx
			.details
			.signature_bytes()
			.expect("DA can only be executed signed")
			.to_vec();
		let app_id = read_app_id(&tx.details).expect("There must be an app id");
		let data = tx.value.data.0.clone();
		Self {
			tx_hash,
			tx_index,
			data,
			tx_signer,
			app_id,
		}
	}

	pub fn from_static_block_transaction(
		tx: StaticBlockTransaction<DataAvailabilityCalls::SubmitData>,
	) -> Self {
		let tx_hash = tx.inner.hash();
		let tx_index = tx.inner.index();
		let tx_signer = tx
			.inner
			.signature_bytes()
			.expect("DA can only be executed signed")
			.to_vec();
		let app_id = read_app_id(&tx.inner).expect("There must be an app id");
		let data = tx.value.data.0.clone();
		Self {
			tx_hash,
			tx_index,
			data,
			tx_signer,
			app_id,
		}
	}

	pub fn to_ascii(&self) -> Option<String> {
		to_ascii(self.data.clone())
	}
}

pub fn to_ascii(value: Vec<u8>) -> Option<String> {
	String::from_utf8(value).ok()
}
