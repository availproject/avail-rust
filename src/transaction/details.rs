use crate::{error::ClientError, utils, AExtrinsicEvents, AOnlineClient, H256};
use std::sync::Arc;
use subxt::{blocks::StaticExtrinsic, events::StaticEvent};

#[derive(Debug, Clone)]
pub struct TransactionDetails {
	pub events: Arc<AExtrinsicEvents>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub block_hash: H256,
	pub block_number: u32,
}

impl TransactionDetails {
	pub fn new(
		events: AExtrinsicEvents,
		tx_hash: H256,
		tx_index: u32,
		block_hash: H256,
		block_number: u32,
	) -> Self {
		Self {
			events: events.into(),
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
    events: ExtrinsicEvents {{
        ext_hash: {:?},
        idx: {},
        events: Events {{
            num_events: {},
            ...
        }},
    }},
    tx_hash: {:?},
    tx_index: {},
    block_hash: {:?},
    block_number: {},
}}
		"#,
			self.events.extrinsic_hash(),
			self.events.extrinsic_index(),
			self.events.all_events_in_block().len(),
			self.tx_hash,
			self.tx_index,
			self.block_hash,
			self.block_number
		);

		println!("{}", formatted_string);
	}

	pub async fn fetch_block(
		&self,
		client: &AOnlineClient,
	) -> Result<crate::block::Block, ClientError> {
		crate::block::Block::new(client, self.block_hash).await
	}

	pub fn find_first_event<T>(&self) -> Option<T>
	where
		T: StaticEvent,
	{
		self.events.find_first::<T>().ok().flatten()
	}

	pub fn find_last_event<T>(&self) -> Option<T>
	where
		T: StaticEvent,
	{
		self.events.find_last::<T>().ok().flatten()
	}

	pub fn find_event<T>(&self) -> Vec<T>
	where
		T: StaticEvent,
	{
		self.events.find::<T>().flatten().collect()
	}

	pub async fn get_data<T>(&self, client: &AOnlineClient) -> Option<T>
	where
		T: StaticExtrinsic,
	{
		let block = self.fetch_block(client).await.ok()?;
		let tx = block.transaction_by_index_static::<T>(self.tx_index)?;
		Some(tx.value)
	}

	pub fn is_successful(&self, client: &AOnlineClient) -> Result<(), subxt::Error> {
		utils::check_if_transaction_was_successful(client, &self.events)
	}
}
