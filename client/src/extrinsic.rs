use crate::{
	Client,
	block::{BEvent, BRxt, BSxt, Block, BlockRawExtrinsic, BlockSignedExtrinsic, ExtrinsicEvents},
	subscription::SubscriptionBuilder,
	subxt_signer::sr25519::Keypair,
	transaction_options::{Options, RefinedMortality, RefinedOptions},
};
use avail_rust_core::{
	AccountId, BlockRef, EncodeSelector, H256, HasHeader, TransactionConvertible,
	ext::codec::Encode,
	extrinsic::{ExtrinsicAdditional, ExtrinsicCall},
	types::{
		metadata::{HashString, TxRef},
		substrate::FeeDetails,
	},
};
use codec::Decode;
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
	pub call: ExtrinsicCall,
}

impl SubmittableTransaction {
	pub fn new(client: Client, call: ExtrinsicCall) -> Self {
		Self { client, call }
	}

	pub async fn sign_and_submit(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<SubmittedTransaction, avail_rust_core::Error> {
		self.client
			.rpc()
			.sign_and_submit_call(signer, &self.call, options)
			.await
	}

	pub async fn sign(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<avail_rust_core::GenericExtrinsic, avail_rust_core::Error> {
		self.client.rpc().sign_call(signer, &self.call, options).await
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

#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub tx_hash: H256,
	pub account_id: AccountId,
	pub options: RefinedOptions,
	pub additional: ExtrinsicAdditional,
}

impl SubmittedTransaction {
	pub fn new(
		client: Client,
		tx_hash: H256,
		account_id: AccountId,
		options: RefinedOptions,
		additional: ExtrinsicAdditional,
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
	pub block_ref: BlockRef,
	pub tx_ref: TxRef,
}

impl TransactionReceipt {
	pub fn new(client: Client, block_ref: BlockRef, tx_ref: TxRef) -> Self {
		Self { client, block_ref, tx_ref }
	}

	pub async fn block_state(&self) -> Result<BlockState, avail_rust_core::Error> {
		self.client.rpc().block_state(self.block_ref).await
	}

	pub async fn tx<T: HasHeader + Decode>(&self) -> Result<BlockSignedExtrinsic<T>, avail_rust_core::Error> {
		let block = BSxt::new(self.client.clone(), self.block_ref.height);
		let tx = block.get(self.tx_ref.index).await?;
		let Some(tx) = tx else {
			return Err("Failed to find transaction".into());
		};

		Ok(tx)
	}

	pub async fn call<T: HasHeader + Decode>(&self) -> Result<T, avail_rust_core::Error> {
		let block = BSxt::new(self.client.clone(), self.block_ref.height);
		let tx = block.get(self.tx_ref.index).await?;
		let Some(tx) = tx else {
			return Err("Failed to find transaction".into());
		};

		Ok(tx.call)
	}

	pub async fn raw_ext(&self, encode_as: EncodeSelector) -> Result<BlockRawExtrinsic, avail_rust_core::Error> {
		let block = BRxt::new(self.client.clone(), self.block_ref.height);
		let ext = block.get(self.tx_ref.index, encode_as).await?;
		let Some(ext) = ext else {
			return Err("Failed to find extrinsic".into());
		};

		Ok(ext)
	}

	pub async fn events(&self) -> Result<ExtrinsicEvents, avail_rust_core::Error> {
		let block = BEvent::new(self.client.clone(), self.block_ref.hash);
		let events = block.ext(self.tx_ref.index).await?;
		let Some(events) = events else {
			return Err("No events were found".into());
		};
		Ok(events)
	}

	pub async fn from_range(
		client: Client,
		tx_hash: impl Into<HashString>,
		block_start: u32,
		block_end: u32,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, avail_rust_core::Error> {
		if block_start > block_end {
			return Err("Block Start cannot start after Block End".into());
		}
		let tx_hash: HashString = tx_hash.into();
		let mut sub = SubscriptionBuilder::default()
			.follow(use_best_block)
			.block_height(block_start)
			.build(&client)
			.await?;

		loop {
			let block_ref = sub.next(&client).await?;

			let block = BRxt::new(client.clone(), block_ref.height);
			let ext = block.get(tx_hash.clone(), EncodeSelector::None).await?;
			if let Some(ext) = ext {
				let tr = TransactionReceipt::new(client.clone(), block_ref, (ext.ext_hash(), ext.ext_index()).into());
				return Ok(Some(tr));
			}

			if block_ref.height >= block_end {
				return Ok(None);
			}
		}
	}
}

pub struct Utils;
impl Utils {
	pub async fn transaction_receipt(
		client: Client,
		tx_hash: H256,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, avail_rust_core::Error> {
		let Some(block_ref) =
			Self::find_correct_block_info(&client, nonce, tx_hash, account_id, mortality, use_best_block).await?
		else {
			return Ok(None);
		};

		let block = Block::new(client.clone(), block_ref.hash);
		let ext = block.rxt.get(tx_hash, EncodeSelector::None).await?;

		let Some(ext) = ext else {
			return Ok(None);
		};

		let tx_ref = TxRef::from((ext.ext_hash(), ext.ext_index()));
		Ok(Some(TransactionReceipt::new(client, block_ref, tx_ref)))
	}

	pub async fn find_correct_block_info(
		client: &Client,
		nonce: u32,
		tx_hash: H256,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<BlockRef>, avail_rust_core::Error> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;

		let mut sub = SubscriptionBuilder::new()
			.block_height(mortality.block_height)
			.follow(use_best_block)
			.build(client)
			.await?;
		let mut current_block_height = mortality.block_height;

		#[cfg(feature = "tracing")]
		{
			match use_best_block {
				true => {
					let info = client.best().block_info().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, info.height, mortality_ends_height);
				},
				false => {
					let info = client.finalized().block_info().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, info.height, mortality_ends_height);
				},
			};
		}

		while mortality_ends_height >= current_block_height {
			let info = sub.next(client).await?;
			current_block_height = sub.current_block_height();

			let state_nonce = client.rpc().block_nonce(account_id, info.hash).await?;
			if state_nonce > nonce {
				trace_new_block(nonce, state_nonce, account_id, info, true);
				return Ok(Some(info));
			}
			if state_nonce == 0 {
				let block = Block::new(client.clone(), info.hash);
				let ext = block.rxt.get(tx_hash, EncodeSelector::None).await?;
				if ext.is_some() {
					trace_new_block(nonce, state_nonce, account_id, info, true);
					return Ok(Some(info));
				}
			}

			trace_new_block(nonce, state_nonce, account_id, info, false);
		}

		Ok(None)
	}
}

fn trace_new_block(nonce: u32, state_nonce: u32, account_id: &AccountId, block_info: BlockRef, search_done: bool) {
	#[cfg(feature = "tracing")]
	{
		if search_done {
			info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done", nonce, account_id, block_info.height, block_info.hash, state_nonce);
		} else {
			info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}.", nonce, account_id, block_info.height, block_info.hash, state_nonce);
		}
	}

	#[cfg(not(feature = "tracing"))]
	{
		let _ = (nonce, state_nonce, account_id, block_info, search_done);
	}
}
