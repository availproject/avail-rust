use crate::{block::EventRecords, block_transaction::Filter, error::ClientError, utils, Client, H256};
use std::sync::Arc;
use subxt::blocks::StaticExtrinsic;

#[derive(Debug, Clone)]
pub struct TransactionDetails {
	client: Client,
	pub events: Option<Arc<EventRecords>>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub block_hash: H256,
	pub block_number: u32,
}

impl TransactionDetails {
	pub fn new(
		client: Client,
		events: Option<EventRecords>,
		tx_hash: H256,
		tx_index: u32,
		block_hash: H256,
		block_number: u32,
	) -> Self {
		let events = events.map(|x| x.into());
		Self {
			client,
			events,
			tx_hash,
			tx_index,
			block_hash,
			block_number,
		}
	}

	/// Returns None if it was not possible to determine if the transaction was successful or not
	/// If Some is returned then
	///    true means the transaction was successful
	///    false means the transaction failed
	pub fn is_successful(&self) -> Option<bool> {
		match &self.events {
			Some(events) => utils::check_if_transaction_was_successful(events),
			None => None,
		}
	}

	/// Returns Err if it was not possible to determine if the transaction was decodable
	/// If Ok is returned then
	///    Some means the transaction was successfully decoded
	///    None means the transaction cannot be decoded as T
	pub async fn decode_as<T: StaticExtrinsic + Clone>(&self) -> Result<Option<T>, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let filter = Filter::new().tx_index(self.tx_index);
		let txs = block.transactions_static::<T>(filter);
		if txs.is_empty() {
			return Ok(None);
		}
		Ok(Some(txs[0].value.clone()))
	}

	/// Returns Err if it was not possible to determine if the transaction was decodable
	/// If Ok is returned then
	///    true means the transaction was successfully decoded
	///    false means the transaction cannot be decoded as T
	pub async fn is<T: StaticExtrinsic + Clone>(&self) -> Result<bool, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let filter = Filter::new().tx_index(self.tx_index);
		let txs = block.transactions_static::<T>(filter);
		Ok(!txs.is_empty())
	}
}
