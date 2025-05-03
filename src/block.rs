use crate::{
	avail::data_availability::calls::types as DataAvailabilityCalls,
	block_transaction::{BlockTransactions, Filter, StaticBlockTransaction},
	error::ClientError,
	primitives::block::extrinsics_params::CheckAppId,
	rpc, ABlock, AEventDetails, AEvents, AExtrinsicDetails, AExtrinsicEvents, AExtrinsics, AccountId, BlockTransaction,
	Client,
};
use codec::Decode;
use primitive_types::H256;
use std::sync::Arc;
use subxt::{
	backend::StreamOfResults,
	blocks::StaticExtrinsic,
	events::StaticEvent,
	storage::StorageKeyValuePair,
	utils::{MultiAddress, Yes},
};

#[derive(Clone)]
pub struct Block {
	pub block: Arc<ABlock>,
	pub transactions: Arc<AExtrinsics>,
}

impl Block {
	pub async fn new(client: &Client, block_hash: H256) -> Result<Self, ClientError> {
		let (block, transactions) = fetch_transactions(client, block_hash).await?;
		Ok(Self {
			block: block.into(),
			transactions: transactions.into(),
		})
	}

	pub async fn new_best_block(client: &Client) -> Result<Self, ClientError> {
		let block_hash = client.best_block_hash().await?;
		Self::new(client, block_hash).await
	}

	pub async fn new_finalized_block(client: &Client) -> Result<Self, ClientError> {
		let block_hash = client.finalized_block_hash().await?;
		Self::new(client, block_hash).await
	}

	pub async fn from_block(block: ABlock) -> Result<Self, subxt::Error> {
		let transactions = block.extrinsics().await?;
		Ok(Self {
			block: block.into(),
			transactions: transactions.into(),
		})
	}

	pub async fn from_block_number(client: &Client, block_number: u32) -> Result<Self, ClientError> {
		let block_hash = rpc::chain::get_block_hash(client, Some(block_number)).await?;
		let Some(block_hash) = block_hash else {
			return Err(subxt::Error::Other("block hash not found".into()).into());
		};

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

	pub fn transactions_static<E: StaticExtrinsic + Clone>(&self, filter: Filter) -> Vec<StaticBlockTransaction<E>> {
		let mut result = Vec::new();

		for atx in self.transactions.iter() {
			if let Some(app_id) = filter.app_id.as_ref() {
				if read_app_id(&atx) != Some(*app_id) {
					continue;
				}
			}

			if let Some(tx_hash) = filter.tx_hash.as_ref() {
				if atx.hash().ne(tx_hash) {
					continue;
				}
			}

			if let Some(tx_index) = filter.tx_index.as_ref() {
				if atx.index().ne(tx_index) {
					continue;
				}
			}

			if let Some(account_id) = filter.tx_signer.as_ref() {
				if read_account_id(&atx) != Some(account_id.clone()) {
					continue;
				}
			}

			let Some(value) = atx.as_extrinsic::<E>().ok().flatten() else {
				continue;
			};

			let value = StaticBlockTransaction { inner: atx, value };
			result.push(value);
		}

		result
	}

	pub fn transactions(&self, filter: Filter) -> BlockTransactions {
		let mut result = Vec::new();

		for atx in self.transactions.iter() {
			if let Some(app_id) = filter.app_id.as_ref() {
				if read_app_id(&atx) != Some(*app_id) {
					continue;
				}
			}

			if let Some(tx_hash) = filter.tx_hash.as_ref() {
				if atx.hash().ne(tx_hash) {
					continue;
				}
			}

			if let Some(tx_index) = filter.tx_index.as_ref() {
				if atx.index().ne(tx_index) {
					continue;
				}
			}

			if let Some(account_id) = filter.tx_signer.as_ref() {
				if read_account_id(&atx) != Some(account_id.clone()) {
					continue;
				}
			}

			let block_tx = BlockTransaction { inner: atx };
			result.push(block_tx);
		}

		BlockTransactions { inner: result }
	}

	pub fn data_submissions(&self, filter: Filter) -> Vec<DataSubmission> {
		self.transactions_static(filter)
			.iter()
			.map(DataSubmission::from_static)
			.collect()
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

	pub async fn storage_iter<T>(&self, address: T) -> Result<StreamOfResults<StorageKeyValuePair<T>>, subxt::Error>
	where
		T: subxt::storage::Address<IsIterable = Yes> + 'static,
		T::Keys: 'static + Sized,
	{
		self.block.storage().iter(address).await
	}

	pub fn transaction_hash_to_index(&self, tx_hash: H256) -> Option<u32> {
		transaction_hash_to_index(&self.transactions, tx_hash).first().cloned()
	}
}

pub async fn fetch_transactions(client: &Client, block_hash: H256) -> Result<(ABlock, AExtrinsics), subxt::Error> {
	let block = client.blocks().at(block_hash).await?;
	let transactions = block.extrinsics().await?;
	Ok((block, transactions))
}

pub async fn fetch_tx_events(transactions: &AExtrinsics, tx_index: u32) -> Result<AExtrinsicEvents, ClientError> {
	let iter = transactions.iter();
	for details in iter {
		if details.index() != tx_index {
			continue;
		}

		return details.events().await.map_err(ClientError::from);
	}

	Err(ClientError::from("Events not found for that transaction index"))
}

pub fn read_app_id(transaction: &AExtrinsicDetails) -> Option<u32> {
	transaction
		.transaction_extensions()?
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
	pub fn from_static(tx: &StaticBlockTransaction<DataAvailabilityCalls::SubmitData>) -> Self {
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
		self.account_id().map(|x| std::format!("{}", x))
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

	// Returns an array of E.
	// Some(Vec<E>) means we were able to decode all E events
	// None means we failed to decode some E events.
	pub fn find_checked<E: StaticEvent>(&self) -> Option<Vec<E>> {
		let mut result: Vec<E> = Vec::new();
		for ev in self.inner.iter() {
			// If result is Err() then we found the tx but failed to decode it
			let decoded = match ev.as_event::<E>() {
				Ok(x) => x,
				Err(_) => return None,
			};

			// If decoded is None then we can skip it as it doesn't have the correct pallet or event name.
			if let Some(decoded) = decoded {
				result.push(decoded);
			}
		}

		Some(result)
	}

	// Return None if the event has not been found.
	// Returns Some(None) if the event has been found but we failed to decode it.
	// Returns Some(E) if the event has been found and we decoded it.
	pub fn find_first<E: StaticEvent>(&self) -> Option<Option<E>> {
		for ev in self.inner.iter() {
			// If result is Err() then we found the tx but failed to decode it
			let decoded = match ev.as_event::<E>() {
				Ok(x) => x,
				Err(_) => return Some(None),
			};

			// If decoded is None then we can skip it as it doesn't have the correct pallet or event name.
			if let Some(decoded) = decoded {
				return Some(Some(decoded));
			}
		}

		None
	}

	// Return None if the event has not been found.
	// Returns Some(None) if the event has been found but we failed to decode it.
	// Returns Some(E) if the event has been found and we decoded it.
	pub fn find_last<E: StaticEvent>(&self) -> Option<Option<E>> {
		for ev in self.inner.iter().rev() {
			// If result is Err() then we found the tx but failed to decode it
			let decoded = match ev.as_event::<E>() {
				Ok(x) => x,
				Err(_) => return Some(None),
			};

			// If decoded is None then we can skip it as it doesn't have the correct pallet or event name.
			if let Some(decoded) = decoded {
				return Some(Some(decoded));
			}
		}

		None
	}

	pub fn has<E: StaticEvent>(&self) -> Option<bool> {
		for ev in self.inner.iter() {
			// If result is Err() then we found the tx but failed to decode it
			let decoded = match ev.as_event::<E>() {
				Ok(x) => x,
				Err(_) => return None,
			};

			// If decoded is None then we can skip it as it doesn't have the correct pallet or event name.
			if decoded.is_some() {
				return Some(true);
			}
		}

		Some(false)
	}

	pub fn count<E: StaticEvent>(&self) -> usize {
		let mut result = 0;
		for ev in self.inner.iter() {
			if ev.pallet_name() != E::PALLET || ev.variant_name() != E::EVENT {
				continue;
			}
			if ev.as_event::<E>().is_ok() {
				result += 1;
			}
		}

		result
	}

	pub fn iter(&self) -> EventRecordsIter {
		EventRecordsIter { inner: self, index: 0 }
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
		if self.inner.inner.is_empty() {
			return None;
		}
		let result = self.inner.inner.remove(0);
		Some(result)
	}
}
