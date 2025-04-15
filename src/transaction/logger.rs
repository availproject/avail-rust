use super::{options::CheckedMortality, watcher::WatcherOptions, Params, TransactionDetails};
use crate::{ABlock, ClientMode};
use log::{info, log_enabled, warn};
use primitive_types::H256;
use std::sync::Arc;
use subxt::tx::DefaultPayload;
use subxt_signer::sr25519::Keypair;

pub struct Logger {
	tx_hash: H256,
	marker: String,
	enabled: bool,
}

impl Logger {
	pub fn new(tx_hash: H256, enabled: bool) -> Arc<Self> {
		let marker = &format!("{:?}", tx_hash)[0..10];
		Self {
			tx_hash,
			marker: String::from(marker),
			enabled,
		}
		.into()
	}

	pub fn enabled(&mut self, value: bool) {
		self.enabled = value
	}

	pub fn is_enabled(&self) -> bool {
		if !log_enabled!(log::Level::Info) || !self.enabled {
			return false;
		}
		true
	}

	pub fn log_watcher_run(
		&self,
		options: &WatcherOptions,
		current_best_height: u32,
		current_finalized_height: u32,
		block_height_timeout: u32,
	) {
		if !self.is_enabled() {
			return;
		}

		info!(target: "watcher", "{}: Watching for Tx Hash: {:?}. Waiting for: {}, Current Best Height: {}, Current Finalized Height: {},  Block height timeout: {:?}, Watcher Mode: {:?}", self.marker, self.tx_hash, options.wait_for.to_str(), current_best_height, current_finalized_height, block_height_timeout, options.mode);
	}

	pub fn log_watcher_new_block(&self, block: &ABlock) {
		if !self.is_enabled() {
			return;
		}

		info!(target: "watcher", "{}: New block fetched. Hash: {:?}, Number: {}", self.marker, block.hash(), block.number());
	}

	pub fn log_watcher_new_block_hash(&self, block_hash: &H256) {
		if !self.is_enabled() {
			return;
		}

		info!(target: "watcher", "{}: New block fetched. Hash: {:?}", self.marker, block_hash);
	}

	pub fn log_watcher_tx_found(&self, details: &TransactionDetails) {
		if !self.is_enabled() {
			return;
		}

		info!(target: "watcher", "{}: Transaction was found. Tx Hash: {:?}, Tx Index: {}, Block Hash: {:?}, Block Number: {}", self.marker, details.tx_hash, details.tx_index, details.block_hash, details.block_number);
	}

	pub fn log_watcher_stop(&self) {
		if !self.is_enabled() {
			return;
		}

		info!(target: "watcher", "{}: No more blocks to search. Failed to find transaction. Tx Hash: {:?}", self.marker, self.tx_hash);
	}

	pub fn log_tx_submitted(&self, keypair: &Keypair, mortality: &CheckedMortality) {
		if !self.is_enabled() {
			return;
		}

		let address = keypair.public_key().to_account_id().to_string();
		let mortality = mortality.block_number + mortality.period as u32;

		info!(
			target: "transaction",
			"{}: Transaction was submitted. Account: {}, TxHash: {:?}, Mortality Block: {:?}",
			self.marker,
			address,
			self.tx_hash,
			mortality
		);
	}

	pub fn log_tx_submitting<T>(&self, keypair: &Keypair, call: &DefaultPayload<T>, params: &Params, mode: ClientMode) {
		if !log_enabled!(log::Level::Info) || !self.enabled {
			return;
		}

		let address = keypair.public_key().to_account_id().to_string();
		let call_name = call.call_name();
		let pallet_name = call.pallet_name();
		let nonce = &params.4 .0;
		let app_id = &params.6 .0;
		info!(
			target: "transaction",
			"Signing and submitting new transaction. Account: {}, Nonce: {:?}, Pallet Name: {}, Call Name: {}, App Id: {}, Client Mode: {:?}",
			address, nonce, pallet_name, call_name, app_id, mode
		);
	}

	pub fn log_tx_retry_abort(&self) {
		warn!(target: "transaction", "{}: Failed to submit and find transaction. Aborting. Tx Hash: {:?}", self.marker, self.tx_hash);
	}

	pub fn log_tx_retry(&self) {
		if !self.is_enabled() {
			return;
		}

		info!(target: "transaction", "{}: Failed to submit and find transaction. Retrying.", self.marker);
	}
}
