use crate::config::MultiAddress;
use crate::config::{AccountId, AccountInfo};
use crate::primitives::TransactionCall;
use codec::Encode;
use primitive_types::H256;
use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
use subxt_core::utils::Yes;

pub trait TxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
}

pub trait TransactionCallLike {
	fn to_call(&self) -> TransactionCall;
}

impl<T: TxDispatchIndex + Encode> TransactionCallLike for T {
	fn to_call(&self) -> TransactionCall {
		TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
	}
}

pub mod data_availability {
	use super::*;
	const PALLET_ID: u8 = 29;

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct CreateApplicationKey {
			pub key: Vec<u8>,
		}
		impl TxDispatchIndex for CreateApplicationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct SubmitData {
			pub data: Vec<u8>,
		}
		impl TxDispatchIndex for SubmitData {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
	}
}

pub mod balances {
	use super::*;
	const PALLET_ID: u8 = 6;

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct TransferAllowDeath {
			pub dest: MultiAddress,
			#[codec(compact)]
			pub amount: u128,
		}
		impl TxDispatchIndex for TransferAllowDeath {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct TransferKeepAlive {
			pub dest: MultiAddress,
			#[codec(compact)]
			pub amount: u128,
		}
		impl TxDispatchIndex for TransferKeepAlive {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Encode)]
		pub struct TransferAll {
			pub dest: MultiAddress,
			pub keep_alive: bool,
		}
		impl TxDispatchIndex for TransferAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod utility {
	use super::*;
	const PALLET_ID: u8 = 1;

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct Batch {
			pub calls: Vec<TransactionCall>,
		}
		impl TxDispatchIndex for Batch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct BatchAll {
			pub calls: Vec<TransactionCall>,
		}
		impl TxDispatchIndex for BatchAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct ForceBatch {
			pub calls: Vec<TransactionCall>,
		}
		impl TxDispatchIndex for ForceBatch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod proxy {
	use super::*;
	const PALLET_ID: u8 = 40;

	pub mod types {
		use super::*;

		#[derive(Debug, Encode, Clone, Copy)]
		#[repr(u8)]
		pub enum ProxyType {
			Any = 0,
			NonTransfer = 1,
			Governance = 2,
			Staking = 3,
			IdentityJudgement = 4,
			NominationPools = 5,
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct Proxy {
			pub id: MultiAddress,
			pub force_proxy_type: Option<super::types::ProxyType>,
			pub call: TransactionCall,
		}
		impl TxDispatchIndex for Proxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct AddProxy {
			pub id: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl TxDispatchIndex for AddProxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Encode)]
		pub struct RemoveProxy {
			pub delegate: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl TxDispatchIndex for RemoveProxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct RemoveProxies;
		impl TxDispatchIndex for RemoveProxies {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Encode)]
		pub struct CreatePure {
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
			pub index: u16,
		}
		impl TxDispatchIndex for CreatePure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Encode)]
		pub struct KillPure {
			pub spawner: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub index: u16,
			#[codec(compact)]
			pub height: u32,
			#[codec(compact)]
			pub ext_index: u32,
		}
		impl TxDispatchIndex for KillPure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
	}
}

pub mod multisig {
	use super::*;
	const PALLET_ID: u8 = 34;

	pub mod types {
		use super::*;
		pub use crate::from_substrate::Weight;

		#[derive(Debug, Encode, Clone, Copy)]
		pub struct Timepoint {
			height: u32,
			index: u32,
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct AsMultiThreshold1 {
			pub other_signatories: Vec<AccountId>,
			pub call: TransactionCall,
		}
		impl TxDispatchIndex for AsMultiThreshold1 {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct AsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call: TransactionCall,
			pub max_weight: super::types::Weight,
		}
		impl TxDispatchIndex for AsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Encode)]
		pub struct ApproveAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call_hash: H256,
			pub max_weight: super::types::Weight,
		}
		impl TxDispatchIndex for ApproveAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct CancelAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: super::types::Timepoint,
			pub call_hash: H256,
		}
		impl TxDispatchIndex for CancelAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
	}
}

pub mod vector {
	use super::*;
	const PALLET_ID: u8 = 39;

	pub mod types {
		use serde::Deserialize;

		use super::*;
		pub use crate::from_substrate::Weight;

		/// Message type used to bridge between Avail & other chains
		#[derive(Debug, Clone, Encode, Deserialize)]
		#[serde(rename_all = "camelCase")]
		pub struct AddressedMessage {
			pub message: Message,
			pub from: H256,
			pub to: H256,
			#[codec(compact)]
			pub origin_domain: u32,
			#[codec(compact)]
			pub destination_domain: u32,
			/// Unique identifier for the message
			#[codec(compact)]
			pub id: u64,
		}

		/// Possible types of Messages allowed by Avail to bridge to other chains.
		#[derive(Debug, Clone, Encode, Deserialize)]
		pub enum Message {
			ArbitraryMessage(Vec<u8>),
			FungibleToken {
				asset_id: H256,
				#[codec(compact)]
				amount: u128,
			},
		}

		#[derive(Debug, Clone, Encode)]
		pub struct Configuration {
			#[codec(compact)]
			pub slots_per_period: u64,
			#[codec(compact)]
			pub finality_threshold: u16,
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct FulfillCall {
			pub function_id: H256,
			pub input: Vec<u8>,
			pub output: Vec<u8>,
			pub proof: Vec<u8>,
			#[codec(compact)]
			pub slot: u64,
		}
		impl TxDispatchIndex for FulfillCall {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct Execute {
			#[codec(compact)]
			pub slot: u64,
			pub addr_message: super::types::AddressedMessage,
			pub account_proof: Vec<Vec<u8>>,
			pub storage_proof: Vec<Vec<u8>>,
		}
		impl TxDispatchIndex for Execute {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Encode)]
		pub struct SourceChainFroze {
			#[codec(compact)]
			pub source_chain_id: u32,
			pub frozen: bool,
		}
		impl TxDispatchIndex for SourceChainFroze {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct SendMessage {
			#[codec(compact)]
			pub slot: u64,
			pub message: super::types::Message,
			pub to: H256,
			#[codec(compact)]
			pub domain: u32,
		}
		impl TxDispatchIndex for SendMessage {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Encode)]
		pub struct SetPoseidonHash {
			#[codec(compact)]
			pub period: u64,
			pub poseidon_hash: Vec<u8>,
		}
		impl TxDispatchIndex for SetPoseidonHash {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Encode)]
		pub struct SetBroadcaster {
			#[codec(compact)]
			pub broadcaster_domain: u32,
			pub broadcaster: H256,
		}
		impl TxDispatchIndex for SetBroadcaster {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 5);
		}

		#[derive(Encode)]
		pub struct SetWhitelistedDomains {
			pub value: Vec<u32>,
		}
		impl TxDispatchIndex for SetWhitelistedDomains {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 6);
		}

		#[derive(Encode)]
		pub struct SetConfiguration {
			pub value: super::types::Configuration,
		}
		impl TxDispatchIndex for SetConfiguration {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 7);
		}

		#[derive(Encode)]
		pub struct SetFunctionIds {
			pub value: Option<(H256, H256)>,
		}
		impl TxDispatchIndex for SetFunctionIds {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 8);
		}

		#[derive(Encode)]
		pub struct SetStepVerificationKey {
			pub value: Option<Vec<u8>>,
		}
		impl TxDispatchIndex for SetStepVerificationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 9);
		}

		#[derive(Encode)]
		pub struct SetRotateVerificationKey {
			pub value: Option<Vec<u8>>,
		}
		impl TxDispatchIndex for SetRotateVerificationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 10);
		}

		#[derive(Encode)]
		pub struct SetUpdater {
			pub updater: H256,
		}
		impl TxDispatchIndex for SetUpdater {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 12);
		}

		#[derive(Encode)]
		pub struct Fulfill {
			pub proof: Vec<u8>,
			pub public_values: Vec<u8>,
		}
		impl TxDispatchIndex for Fulfill {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 13);
		}

		#[derive(Encode)]
		pub struct SetSp1VerificationKey {
			pub sp1_vk: H256,
		}
		impl TxDispatchIndex for SetSp1VerificationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 14);
		}

		#[derive(Encode)]
		pub struct SetSyncCommitteeHash {
			pub period: u64,
			pub hash: H256,
		}
		impl TxDispatchIndex for SetSyncCommitteeHash {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 15);
		}

		#[derive(Encode)]
		pub struct EnableMock {
			pub value: bool,
		}
		impl TxDispatchIndex for EnableMock {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 16);
		}

		#[derive(Encode)]
		pub struct MockFulfill {
			pub public_values: Vec<u8>,
		}
		impl TxDispatchIndex for MockFulfill {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 17);
		}
	}
}

pub mod system {
	use super::*;
	const PALLET_ID: u8 = 0;

	pub mod storage {
		use super::*;
		pub fn account(
			account_id: &AccountId,
		) -> StaticAddress<StaticStorageKey<AccountId>, AccountInfo, Yes, Yes, ()> {
			let address = StaticAddress::new_static(
				"System",
				"Account",
				StaticStorageKey::new(account_id),
				Default::default(),
			);
			address.unvalidated()
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct Remark {
			pub remark: Vec<u8>,
		}
		impl TxDispatchIndex for Remark {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct SetCode {
			pub code: Vec<u8>,
		}
		impl TxDispatchIndex for SetCode {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct SetCodeWithoutChecks {
			pub code: Vec<u8>,
		}
		impl TxDispatchIndex for SetCodeWithoutChecks {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Encode)]
		pub struct RemarkWithEvent {
			pub remark: Vec<u8>,
		}
		impl TxDispatchIndex for RemarkWithEvent {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 7);
		}
	}
}

pub mod utils {
	use std::array::TryFromSliceError;

	#[derive(Debug, Clone)]
	pub struct SessionKeys {
		pub babe: [u8; 32],
		pub grandpa: [u8; 32],
		pub im_online: [u8; 32],
		pub authority_discovery: [u8; 32],
	}

	impl TryFrom<&[u8]> for SessionKeys {
		type Error = String;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			if value.len() != 128 {
				return Err(String::from(
					"Session keys len cannot have length be more or less than 128",
				));
			}

			let err = |e: TryFromSliceError| e.to_string();

			let babe: [u8; 32] = value[0..32].try_into().map_err(err)?;
			let grandpa: [u8; 32] = value[32..64].try_into().map_err(err)?;
			let im_online: [u8; 32] = value[64..96].try_into().map_err(err)?;
			let authority_discovery: [u8; 32] = value[96..128].try_into().map_err(err)?;
			Ok(Self {
				babe,
				grandpa,
				im_online,
				authority_discovery,
			})
		}
	}
}
