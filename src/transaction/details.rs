use subxt::blocks::StaticExtrinsic;

use crate::{block::EventRecords, error::ClientError, utils, Client, H256};
use std::sync::Arc;

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
		let events = match events {
			Some(x) => Some(x.into()),
			None => None,
		};
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
	/// 	Ok means the transaction was successful
	/// 	Err means the transaction failed
	pub fn is_successful(&self) -> Option<bool> {
		match &self.events {
			Some(events) => utils::check_if_transaction_was_successful(events),
			None => None,
		}
	}

	pub async fn decode_as<T: StaticExtrinsic>(&self) -> Result<Option<T>, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let tx = block.transaction_by_index_static::<T>(self.tx_index);
		match tx {
			Some(x) => Ok(Some(x.value)),
			None => Ok(None),
		}
	}

	pub async fn is<T: StaticExtrinsic>(&self) -> Result<bool, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let tx = block.transaction_by_index_static::<T>(self.tx_index);
		match tx {
			Some(_) => Ok(true),
			None => Ok(false),
		}
	}
}
