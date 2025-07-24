use crate::{
	Client,
	subscription::{HeaderSubscription, Subscriber},
	subxt_signer::sr25519::Keypair,
	transaction_options::{Options, RefinedMortality, RefinedOptions},
};
use avail_rust_core::{
	AccountId, BlockLocation, H256, TransactionConvertible,
	config::TransactionLocation,
	ext::codec::Encode,
	from_substrate::FeeDetails,
	rpc::system::fetch_events_v1_types::RuntimeEvent,
	transaction::{TransactionAdditional, TransactionCall},
};
#[cfg(feature = "tracing")]
use tracing::info;

pub trait SubmittableTransactionLike {
	fn to_submittable(&self, client: Client) -> SubmittableTransaction;
}

impl<T: TransactionConvertible> SubmittableTransactionLike for T {
	fn to_submittable(&self, client: Client) -> SubmittableTransaction {
		let call = self.to_call();
		SubmittableTransaction::new(client, call)
	}
}

#[derive(Clone)]
pub struct SubmittableTransaction {
	client: Client,
	pub call: TransactionCall,
}

impl SubmittableTransaction {
	pub fn new(client: Client, call: TransactionCall) -> Self {
		Self { client, call }
	}

	pub async fn sign_and_submit(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<SubmittedTransaction, avail_rust_core::Error> {
		self.client.sign_and_submit_call(signer, &self.call, options).await
	}

	pub async fn sign(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<avail_rust_core::Transaction, avail_rust_core::Error> {
		self.client.sign_call(signer, &self.call, options).await
	}

	pub async fn estimate_call_fees(&self, at: Option<H256>) -> Result<FeeDetails, avail_rust_core::Error> {
		let call = self.call.encode();
		self.client
			.runtime_api()
			.transaction_payment_query_call_fee_details(call, at)
			.await
	}

	pub async fn estimate_extrinsic_fees(
		&self,
		signer: &Keypair,
		options: Options,
		at: Option<H256>,
	) -> Result<FeeDetails, avail_rust_core::Error> {
		let transaction = self.sign(signer, options).await?;
		let transaction = transaction.encode();
		self.client
			.runtime_api()
			.transaction_payment_query_fee_details(transaction, at)
			.await
	}
}

#[derive(Clone, Copy)]
pub enum ReceiptMethod {
	Default { use_best_block: bool },
}

impl Default for ReceiptMethod {
	fn default() -> Self {
		Self::Default { use_best_block: false }
	}
}

#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub tx_hash: H256,
	pub account_id: AccountId,
	pub options: RefinedOptions,
	pub additional: TransactionAdditional,
}

impl SubmittedTransaction {
	pub fn new(
		client: Client,
		tx_hash: H256,
		account_id: AccountId,
		options: RefinedOptions,
		additional: TransactionAdditional,
	) -> Self {
		Self { client, tx_hash, account_id, options, additional }
	}

	pub async fn receipt(&self, use_best_block: bool) -> Result<Option<TransactionReceipt>, avail_rust_core::Error> {
		Utils::transaction_receipt(
			self.client.clone(),
			self.tx_hash,
			self.options.nonce,
			&self.account_id,
			&self.options.mortality,
			use_best_block,
		)
		.await
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockState {
	Included = 0,
	Finalized = 1,
	Discarded = 2,
	DoesNotExist = 3,
}

#[derive(Clone)]
pub struct TransactionReceipt {
	client: Client,
	pub block_loc: BlockLocation,
	pub tx_loc: TransactionLocation,
}

impl TransactionReceipt {
	pub fn new(client: Client, block_loc: BlockLocation, tx_loc: TransactionLocation) -> Self {
		Self { client, block_loc, tx_loc }
	}

	pub async fn block_state(&self) -> Result<BlockState, avail_rust_core::Error> {
		self.client.block_state(self.block_loc).await
	}

	pub async fn tx_events(&self) -> Result<Vec<RuntimeEvent>, avail_rust_core::Error> {
		let events_client = self.client.event_client();
		let Some(events) = events_client
			.transaction_events(self.tx_loc.index, true, false, self.block_loc.hash)
			.await?
		else {
			return Err("Failed to to find events".into());
		};

		Ok(events)
	}
}

pub struct Utils;
impl Utils {
	/// TODO
	pub async fn transaction_receipt(
		client: Client,
		tx_hash: H256,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, avail_rust_core::Error> {
		let Some(block_loc) =
			Self::find_block_loc_via_nonce(&client, nonce, account_id, mortality, use_best_block).await?
		else {
			return Ok(None);
		};

		let block_client = client.block_client();
		let tx = block_client
			.transaction_with_retries(block_loc.hash.into(), tx_hash.into(), None)
			.await?;

		let Some(tx) = tx else {
			return Ok(None);
		};

		let tx_location = TransactionLocation::from((tx.tx_hash, tx.tx_index));

		Ok(Some(TransactionReceipt::new(client, block_loc, tx_location)))
	}

	/// TODO
	pub async fn find_block_loc_via_nonce(
		client: &Client,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<BlockLocation>, avail_rust_core::Error> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;
		let sub = match use_best_block {
			true => Subscriber::new_best_block(3_000, mortality.block_height),
			false => Subscriber::new_finalized_block(3_000, mortality.block_height),
		};
		let mut sub = HeaderSubscription::new(client.clone(), sub);
		let mut current_block_height = mortality.block_height;

		#[cfg(feature = "tracing")]
		{
			match use_best_block {
				true => {
					let id = client.best_block_loc().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, id.height, mortality_ends_height);
				},
				false => {
					let id = client.finalized_block_loc().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, id.height, mortality_ends_height);
				},
			};
		}

		while mortality_ends_height >= current_block_height {
			let next_header = sub.next().await?;
			current_block_height = sub.current_block_height();

			let Some(header) = next_header else {
				continue;
			};
			let block_loc = BlockLocation::from((header.hash(), header.number));
			let state_nonce = client.block_nonce(account_id, block_loc.hash).await?;

			if state_nonce > nonce {
				trace_new_block(nonce, state_nonce, account_id, block_loc, true);
				return Ok(Some(block_loc));
			}

			trace_new_block(nonce, state_nonce, account_id, block_loc, false);
		}

		Ok(None)
	}
}

#[cfg(feature = "tracing")]
fn trace_new_block(nonce: u32, state_nonce: u32, account_id: &AccountId, block_loc: BlockLocation, search_done: bool) {
	if search_done {
		info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done", nonce, account_id, block_loc.height, block_loc.hash, state_nonce);
	} else {
		info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}.", nonce, account_id, block_loc.height, block_loc.hash, state_nonce);
	}
}

#[cfg(not(feature = "tracing"))]
fn trace_new_block(
	_nonce: u32,
	_state_nonce: u32,
	_account_id: &AccountId,
	_block_loc: BlockLocation,
	_search_done: bool,
) {
	return;
}
