use crate::{avail::runtime_types::da_runtime::primitives::SessionKeys, AppUncheckedExtrinsic};
use primitive_types::H256;
use subxt::backend::legacy::rpc_methods::Bytes;

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
