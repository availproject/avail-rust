use crate::{error::ClientError, utils, AExtrinsicEvents, Client, H256};
use std::sync::Arc;
use subxt::{blocks::StaticExtrinsic, events::StaticEvent};

#[derive(Debug, Clone)]
pub struct TransactionDetails {
	pub events: Option<Arc<AExtrinsicEvents>>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub block_hash: H256,
	pub block_number: u32,
}

impl TransactionDetails {
	pub fn new(
		events: Option<AExtrinsicEvents>,
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
			events,
			tx_hash,
			tx_index,
			block_hash,
			block_number,
		}
	}

	pub fn print_debug(&self) {
		let formatted_string = format!(
			r#"
TransactionDetails {{
    tx_hash: {:?},
    tx_index: {},
    block_hash: {:?},
    block_number: {},
}}
		"#,
			self.tx_hash, self.tx_index, self.block_hash, self.block_number
		);

		println!("{}", formatted_string);
	}

	pub async fn fetch_block(&self, client: &Client) -> Result<crate::block::Block, ClientError> {
		crate::block::Block::new(client, self.block_hash).await
	}

	/// Returns None if the event was not found OR if it was not possible to decode the events
	/// Returns Some if the event was found
	///
	/// If .events are None then we are not able to decode events and we don't have access to any event
	pub fn find_first_event<T>(&self) -> Option<T>
	where
		T: StaticEvent,
	{
		match &self.events {
			Some(events) => events.find_first::<T>().ok().flatten(),
			None => None,
		}
	}

	/// Returns None if the event was not found OR if it was not possible to decode the events
	/// Returns Some if the event was found
	///
	/// If .events are None then we are not able to decode events and we don't have access to any event
	pub fn find_last_event<T>(&self) -> Option<T>
	where
		T: StaticEvent,
	{
		match &self.events {
			Some(events) => events.find_last::<T>().ok().flatten(),
			None => None,
		}
	}

	/// Returns an empty array if the event was not found OR if it was not possible to decode the events
	/// Returns an array with at least one element if at least one event was found
	///
	/// If .events are None then we are not able to decode events and we don't have access to any event
	pub fn find_event<T>(&self) -> Vec<T>
	where
		T: StaticEvent,
	{
		match &self.events {
			Some(events) => events.find::<T>().flatten().collect(),
			None => vec![],
		}
	}

	/// Returns None if we failed to get the call data OR if it was not possible to get the block that contains the call data
	/// Returns Some if we managed to get the call data
	pub async fn get_call_data<T>(&self, client: &Client) -> Option<T>
	where
		T: StaticExtrinsic,
	{
		let block = self.fetch_block(client).await.ok()?;
		let tx = block.transaction_by_index_static::<T>(self.tx_index)?;
		Some(tx.value)
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
}
