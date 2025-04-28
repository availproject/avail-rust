use crate::{avail::runtime_types::da_runtime::primitives::SessionKeys, block::EventRecords, AppUncheckedExtrinsic};
use primitive_types::H256;
use subxt::backend::legacy::rpc_methods::Bytes;

/// Returns Ok if the transaction was successful
/// Returns Err if the transaction failed
pub fn check_if_transaction_was_successful(events: &EventRecords) -> Option<bool> {
	// Try to find any errors; return the first one we encounter.
	for ev in events.iter() {
		if ev.pallet_name() != "System" {
			continue;
		}

		if ev.variant_name() == "ExtrinsicFailed" {
			return Some(false);
		}

		if ev.variant_name() == "ExtrinsicSuccess" {
			return Some(true);
		}
	}

	None
}

pub fn decode_raw_block_rpc_extrinsics(extrinsics: Vec<Bytes>) -> Result<Vec<AppUncheckedExtrinsic>, String> {
	let extrinsics: Result<Vec<AppUncheckedExtrinsic>, String> =
		extrinsics.into_iter().map(AppUncheckedExtrinsic::try_from).collect();

	extrinsics
}

pub fn deconstruct_session_keys(session_keys: Vec<u8>) -> Result<SessionKeys, String> {
	use crate::avail::runtime_types::{
		pallet_im_online, sp_authority_discovery, sp_consensus_babe, sp_consensus_grandpa,
		sp_core::{ed25519::Public as EDPublic, sr25519::Public as SRPublic},
	};
	use core::array::TryFromSliceError;

	if session_keys.len() != 128 {
		return Err(String::from(
			"Session keys len cannot have length be more or less than 128",
		));
	}

	let err = |e: TryFromSliceError| e.to_string();

	let babe: [u8; 32] = session_keys[0..32].try_into().map_err(err)?;
	let grandpa: [u8; 32] = session_keys[32..64].try_into().map_err(err)?;
	let im_online: [u8; 32] = session_keys[64..96].try_into().map_err(err)?;
	let authority_discovery: [u8; 32] = session_keys[96..128].try_into().map_err(err)?;

	Ok(SessionKeys {
		babe: sp_consensus_babe::app::Public(SRPublic(babe)),
		grandpa: sp_consensus_grandpa::app::Public(EDPublic(grandpa)),
		im_online: pallet_im_online::sr25519::app_sr25519::Public(SRPublic(im_online)),
		authority_discovery: sp_authority_discovery::app::Public(SRPublic(authority_discovery)),
	})
}

pub fn deconstruct_session_keys_string(session_keys: String) -> Result<SessionKeys, String> {
	if session_keys.len() != 256 {
		return Err(String::from(
			"Session keys len cannot have length be more or less than 256",
		));
	}

	let err = || String::from("Internal Math Error");
	let len = session_keys.len();
	let mut session_keys_u8: Vec<u8> = Vec::with_capacity(128);
	let mut iter = session_keys.chars();
	for _ in (0..len).step_by(2) {
		let value_1: u8 = iter
			.next()
			.and_then(|v| v.to_digit(16))
			.map(|v| (v * 16) as u8)
			.ok_or_else(err)?;
		let value_2: u8 = iter
			.next()
			.and_then(|v| v.to_digit(16))
			.map(|v| v as u8)
			.ok_or_else(err)?;
		session_keys_u8.push(value_1 + value_2);
	}

	if session_keys_u8.len() != 128 {
		return Err(String::from(
			"Something went wrong and the length of the calculated session keys is wrong",
		));
	}

	deconstruct_session_keys(session_keys_u8)
}

pub trait H256Utils {
	fn from_hex(s: &str) -> Result<H256, String>;
}

impl H256Utils for H256 {
	fn from_hex(s: &str) -> Result<H256, String> {
		let mut s = s;
		if s.starts_with("0x") {
			s = &s[2..];
		}

		if s.len() != 64 {
			let msg = std::format!(
				"Failed to convert string to H256. Expected 64 bytes got {}. Input string: {}",
				s.len(),
				s
			);
			return Err(msg);
		}

		let block_hash = hex::decode(s).map_err(|e| e.to_string())?;
		let block_hash = TryInto::<[u8; 32]>::try_into(block_hash);
		match block_hash {
			Ok(v) => Ok(H256(v)),
			Err(e) => {
				let msg = std::format!("Failed to covert decoded string to H256. Input {:?}", e);
				Err(msg)
			},
		}
	}
}
