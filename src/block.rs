use crate::{
	avail::data_availability::calls::types as DataAvailabilityCalls,
	block_transaction::{BlockTransactions, StaticBlockTransaction},
	error::ClientError,
	primitives::block::extrinsics_params::CheckAppId,
	rpc, ABlock, AEventDetails, AEvents, AExtrinsicDetails, AExtrinsicEvents, AExtrinsics,
	AFoundExtrinsic, AccountId, BlockTransaction, Client,
};
use codec::Decode;
use primitive_types::H256;
use subxt::{
	backend::StreamOfResults,
	blocks::StaticExtrinsic,
	events::StaticEvent,
	storage::StorageKeyValuePair,
	utils::{MultiAddress, Yes},
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

	pub async fn events(&self) -> Option<EventRecords> {
		let events = self.block.events().await.ok()?;
		EventRecords::new(events)
	}

	pub async fn tx_events(&self, tx_index: u32) -> Option<EventRecords> {
		let events = fetch_tx_events(&self.transactions, tx_index).await.ok()?;
		EventRecords::new_ext(events)
	}

	pub fn transaction_count(&self) -> usize {
		self.transactions.len()
	}

	pub fn transaction_all_static<E: StaticExtrinsic>(&self) -> Vec<StaticBlockTransaction<E>> {
		transaction_all_static::<E>(&self.transactions)
	}

	pub fn transaction_by_signer_static<E: StaticExtrinsic>(
		&self,
		account_id: AccountId,
	) -> Vec<StaticBlockTransaction<E>> {
		self.transactions
			.iter()
			.flat_map(|details| {
				let tx_account_id = read_account_id(&details);
				if tx_account_id != Some(account_id.clone()) {
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

	pub fn transaction_all(&self) -> BlockTransactions {
		let txs = self
			.transactions
			.iter()
			.map(|tx| BlockTransaction { inner: tx })
			.collect();
		BlockTransactions { inner: txs }
	}

	pub fn transaction_by_hash(&self, tx_hash: H256) -> Option<BlockTransaction> {
		transaction_by_hash(&self.transactions, tx_hash)
	}

	pub fn transaction_by_signer(&self, account_id: AccountId) -> BlockTransactions {
		let txs = self
			.transactions
			.iter()
			.filter(|tx| read_account_id(tx) == Some(account_id.clone()))
			.map(|tx| BlockTransaction { inner: tx })
			.collect();
		BlockTransactions { inner: txs }
	}

	pub fn transaction_by_index(&self, tx_index: u32) -> Option<BlockTransaction> {
		transaction_by_index(&self.transactions, tx_index)
	}

	pub fn transaction_by_app_id(&self, app_id: u32) -> BlockTransactions {
		let txs = self
			.transactions
			.iter()
			.filter(|tx| read_app_id(tx) == Some(app_id))
			.map(|tx| BlockTransaction { inner: tx })
			.collect();
		BlockTransactions { inner: txs }
	}

	pub fn data_submissions_all(&self) -> Vec<DataSubmission> {
		data_submissions_all(&self.transactions)
	}

	pub fn data_submissions_by_signer(&self, account_id: AccountId) -> Vec<DataSubmission> {
		self.transaction_by_signer_static::<DataAvailabilityCalls::SubmitData>(account_id)
			.into_iter()
			.map(DataSubmission::from_static_block_transaction)
			.collect()
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

pub async fn fetch_tx_events(
	transactions: &AExtrinsics,
	tx_index: u32,
) -> Result<AExtrinsicEvents, ClientError> {
	let iter = transactions.iter();
	for details in iter {
		if details.index() != tx_index {
			continue;
		}

		return details.events().await.map_err(ClientError::from);
	}

	Err(ClientError::from(
		"Events not found for that transaction index",
	))
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

pub fn read_multi_address(transaction: &AExtrinsicDetails) -> Option<MultiAddress<AccountId, u32>> {
	let mut address_bytes = match transaction.address_bytes() {
		Some(x) => x,
		None => return None,
	};

	match MultiAddress::<AccountId, u32>::decode(&mut address_bytes) {
		Ok(x) => Some(x),
		Err(_) => None,
	}
}

pub fn read_account_id(transaction: &AExtrinsicDetails) -> Option<AccountId> {
	match read_multi_address(transaction) {
		Some(MultiAddress::Id(x)) => Some(x),
		_ => None,
	}
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
	pub tx_signer: MultiAddress<AccountId, u32>,
	pub app_id: u32,
}

impl DataSubmission {
	pub fn from_static(tx: AFoundExtrinsic<DataAvailabilityCalls::SubmitData>) -> Self {
		let tx_hash = tx.details.hash();
		let tx_index = tx.details.index();
		let tx_signer = read_multi_address(&tx.details).expect("There must be an address");
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
		let tx_signer = read_multi_address(&tx.inner).expect("There must be an address");
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

	pub fn account_id(&self) -> Option<AccountId> {
		match &self.tx_signer {
			MultiAddress::Id(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn ss58address(&self) -> Option<String> {
		match self.account_id() {
			Some(x) => Some(std::format!("{}", x)),
			_ => None,
		}
	}
}

pub fn to_ascii(value: Vec<u8>) -> Option<String> {
	String::from_utf8(value).ok()
}

#[derive(Debug, Clone)]
pub struct EventRecords {
	pub inner: Vec<AEventDetails>,
}

impl EventRecords {
	pub fn new(events: AEvents) -> Option<Self> {
		let events: Result<Vec<AEventDetails>, _> = events.iter().collect();
		if let Ok(events) = events {
			return Some(EventRecords { inner: events });
		}

		None
	}

	pub fn new_ext(events: AExtrinsicEvents) -> Option<Self> {
		let events: Result<Vec<AEventDetails>, _> = events.iter().collect();
		if let Ok(events) = events {
			return Some(EventRecords { inner: events });
		}

		None
	}

	pub fn len(&self) -> usize {
		self.inner.len()
	}

	pub fn find<E: StaticEvent>(&self) -> Vec<E> {
		let mut result: Vec<E> = Vec::new();
		for ev in self.inner.iter() {
			let event = ev.as_event::<E>();
			if let Some(event) = event.ok().flatten() {
				result.push(event);
			}
		}

		result
	}

	pub fn find_first<E: StaticEvent>(&self) -> Result<Option<E>, subxt_core::Error> {
		for ev in self.inner.iter() {
			if ev.pallet_name() != E::PALLET || ev.variant_name() != E::EVENT {
				continue;
			}
			return ev.as_event::<E>();
		}

		Ok(None)
	}

	pub fn find_last<E: StaticEvent>(&self) -> Result<Option<E>, subxt_core::Error> {
		for ev in self.inner.iter().rev() {
			if ev.pallet_name() != E::PALLET || ev.variant_name() != E::EVENT {
				continue;
			}
			return ev.as_event::<E>();
		}

		Ok(None)
	}

	pub fn iter(&self) -> EventRecordsIter {
		EventRecordsIter {
			inner: self,
			index: 0,
		}
	}
}

impl IntoIterator for EventRecords {
	type Item = AEventDetails;
	type IntoIter = EventRecordsIntoIter;

	fn into_iter(self) -> Self::IntoIter {
		EventRecordsIntoIter { inner: self }
	}
}

pub struct EventRecordsIter<'a> {
	pub inner: &'a EventRecords,
	index: usize,
}

impl<'a> Iterator for EventRecordsIter<'a> {
	type Item = &'a AEventDetails;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.inner.inner.len() {
			let result = Some(&self.inner.inner[self.index]);
			self.index += 1;
			result
		} else {
			None
		}
	}
}

pub struct EventRecordsIntoIter {
	pub inner: EventRecords,
}

impl Iterator for EventRecordsIntoIter {
	type Item = AEventDetails;

	fn next(&mut self) -> Option<Self::Item> {
		if self.inner.inner.len() == 0 {
			return None;
		}
		let result = self.inner.inner.remove(0);
		Some(result)
	}
}
