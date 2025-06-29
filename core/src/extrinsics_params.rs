// Copyright 2019-2024 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

use crate::Era;
use codec::{Compact, Encode};
use scale_info::PortableRegistry;
use subxt_core::{
	client::ClientState,
	config::{Config, ExtrinsicParams, ExtrinsicParamsEncoder, Header, transaction_extensions},
	error::ExtrinsicParamsError,
};

use crate::AppId;

#[derive(Debug, Clone, Copy, Default)]
pub struct CheckAppId(pub AppId);

/// Ideally, we would use avail_core::AppId but we cannot define `RefineParams` for it so we need a wrapper. Crazy
impl<T: Config> transaction_extensions::Params<T> for AppId {}
impl<T: Config> transaction_extensions::Params<T> for CheckAppId {}

impl ExtrinsicParamsEncoder for CheckAppId {
	fn encode_value_to(&self, v: &mut Vec<u8>) {
		Compact::<u32>(self.0.0).encode_to(v);
	}

	fn encode_implicit_to(&self, _: &mut Vec<u8>) {}
}

impl<T: Config> subxt_core::config::ExtrinsicParams<T> for CheckAppId {
	type Params = AppId;

	fn new(_client: &ClientState<T>, id: Self::Params) -> Result<Self, ExtrinsicParamsError> {
		Ok(CheckAppId(id))
	}
}

impl<T: Config> transaction_extensions::TransactionExtension<T> for CheckAppId {
	type Decoded = Compact<u32>;

	fn matches(identifier: &str, _type_id: u32, _types: &PortableRegistry) -> bool {
		identifier == "CheckAppId"
	}
}

/// Type used only for decoding extrinsic from blocks.
pub type OnlyCodecExtra = (
	(),            // CheckNonZeroSender,
	(),            // CheckSpecVersion<Runtime>,
	(),            // CheckTxVersion<Runtime>,
	(),            // CheckGenesis<Runtime>,
	Era,           // CheckEra<Runtime>,
	Compact<u32>,  // CheckNonce<Runtime>,
	(),            // CheckWeight<Runtime>,
	Compact<u128>, // ChargeTransactionPayment<Runtime>,
	AppId,         // CheckAppId<Runtime>,
);

/// The default [`super::ExtrinsicParams`] implementation understands common signed extensions
/// and how to apply them to a given chain.
pub type DefaultExtrinsicParams<T> = transaction_extensions::AnyOf<
	T,
	(
		transaction_extensions::CheckSpecVersion,
		transaction_extensions::CheckTxVersion,
		transaction_extensions::CheckGenesis<T>,
		transaction_extensions::CheckMortality<T>,
		transaction_extensions::CheckNonce,
		transaction_extensions::ChargeTransactionPayment,
		CheckAppId,
	),
>;

/// A builder that outputs the set of [`super::ExtrinsicParams::Params`] required for
/// [`DefaultExtrinsicParams`]. This may expose methods that aren't applicable to the current
/// chain; such values will simply be ignored if so.
pub struct DefaultExtrinsicParamsBuilder<T: Config> {
	/// `None` means the tx will be immortal.
	mortality: Option<Mortality<T::Hash>>,
	/// `None` means the nonce will be automatically set.
	nonce: Option<u64>,
	tip: u128,
	app_id: AppId,
}

#[derive(Debug, Clone)]
struct Mortality<Hash> {
	/// Block hash that mortality starts from
	checkpoint_hash: Hash,
	/// Block number that mortality starts from (must
	// point to the same block as the hash above)
	checkpoint_number: u64,
	/// How many blocks the tx is mortal for
	period: u64,
}

impl<T: Config> Default for DefaultExtrinsicParamsBuilder<T> {
	fn default() -> Self {
		Self {
			mortality: None,
			tip: 0,
			nonce: None,
			app_id: AppId::default(),
		}
	}
}

impl<T: Config> DefaultExtrinsicParamsBuilder<T> {
	/// Configure new extrinsic params. We default to providing no tip
	/// and using an immortal transaction unless otherwise configured
	pub fn new() -> Self {
		Default::default()
	}

	/// Make the transaction mortal, given a block header that it should be mortal from,
	/// and the number of blocks (roughly; it'll be rounded to a power of two) that it will
	/// be mortal for.
	pub fn mortal(mut self, from_block: &T::Header, for_n_blocks: u64) -> Self {
		self.mortality = Some(Mortality {
			checkpoint_hash: from_block.hash(),
			checkpoint_number: from_block.number().into(),
			period: for_n_blocks,
		});
		self
	}

	/// Provide a specific nonce for the submitter of the extrinsic
	pub fn nonce(mut self, nonce: u64) -> Self {
		self.nonce = Some(nonce);
		self
	}

	/// App Id
	pub fn app_id(mut self, app_id: u32) -> Self {
		self.app_id = AppId(app_id);
		self
	}

	/// Make the transaction mortal, given a block number and block hash (which must both point to
	/// the same block) that it should be mortal from, and the number of blocks (roughly; it'll be
	/// rounded to a power of two) that it will be mortal for.
	///
	/// Prefer to use [`DefaultExtrinsicParamsBuilder::mortal()`], which ensures that the block hash
	/// and number align.
	pub fn mortal_unchecked(mut self, from_block_number: u64, from_block_hash: T::Hash, for_n_blocks: u64) -> Self {
		self.mortality = Some(Mortality {
			checkpoint_hash: from_block_hash,
			checkpoint_number: from_block_number,
			period: for_n_blocks,
		});
		self
	}

	/// Provide a tip to the block author in the chain's native token.
	pub fn tip(mut self, tip: u128) -> Self {
		self.tip = tip;
		self
	}

	/// Build the extrinsic parameters.
	pub fn build(self) -> <DefaultExtrinsicParams<T> as ExtrinsicParams<T>>::Params {
		let check_mortality_params = if let Some(mortality) = self.mortality {
			transaction_extensions::CheckMortalityParams::mortal_from_unchecked(
				mortality.period,
				mortality.checkpoint_number,
				mortality.checkpoint_hash,
			)
		} else {
			transaction_extensions::CheckMortalityParams::immortal()
		};

		let charge_transaction_params = transaction_extensions::ChargeTransactionPaymentParams::tip(self.tip);

		let check_nonce_params = match self.nonce {
			Some(x) => transaction_extensions::CheckNonceParams::with_nonce(x),
			None => transaction_extensions::CheckNonceParams::from_chain(),
		};
		(
			(),
			(),
			(),
			check_mortality_params,
			check_nonce_params,
			charge_transaction_params,
			self.app_id,
		)
	}
}
