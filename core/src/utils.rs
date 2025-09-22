use crate::{AccountId, AccountIdLike};
use codec::{Decode, Encode};
use sp_crypto_hashing::blake2_256;

pub fn decode_already_decoded<I: codec::Input>(input: &mut I) -> Result<Vec<u8>, codec::Error> {
	let length = input.remaining_len()?;
	let Some(length) = length else {
		return Err("Failed to decode transaction".into());
	};
	if length == 0 {
		return Ok(Vec::new());
	}
	let mut value = vec![0u8; length];
	input.read(&mut value)?;
	Ok(value)
}

pub fn account_id_from_str(value: &str) -> Result<AccountId, String> {
	if value.starts_with("0x") {
		// Cannot be in SS58 format
		let decoded = const_hex::decode(value.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		return account_id_from_slice(&decoded);
	}

	value.parse().map_err(|e| std::format!("{:?}", e))
}

pub fn account_id_from_slice(value: &[u8]) -> Result<AccountId, String> {
	let account_id: [u8; 32] = match value.try_into() {
		Ok(x) => x,
		Err(err) => return Err(err.to_string()),
	};

	Ok(AccountId { 0: account_id })
}

/// Derive a multi-account ID from the sorted list of accounts and the threshold that are
/// required.
pub fn multi_account_id(who: &[impl Into<AccountIdLike> + Clone], threshold: u16) -> AccountId {
	let who: Vec<AccountIdLike> = who.into_iter().map(|x| x.clone().into()).collect();
	let mut who: Vec<AccountId> = who
		.into_iter()
		.map(|x| AccountId::try_from(x).expect("Malformed string is passed for AccountId"))
		.collect();

	who.sort_by(|x, y| x.cmp(&y));

	let entropy = (b"modlpy/utilisuba", who, threshold).using_encoded(blake2_256);
	Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
		.expect("infinite length input; no invalid inputs for type; qed")
}

/// Input that adds infinite number of zero after wrapped input.
struct TrailingZeroInput<'a>(&'a [u8]);

impl<'a> TrailingZeroInput<'a> {
	/// Create a new instance from the given byte array.
	pub fn new(data: &'a [u8]) -> Self {
		Self(data)
	}

	// Create a new instance which only contains zeroes as input.
	// pub fn zeroes() -> Self {
	// 	Self::new(&[][..])
	// }
}

impl<'a> codec::Input for TrailingZeroInput<'a> {
	fn remaining_len(&mut self) -> Result<Option<usize>, codec::Error> {
		Ok(None)
	}

	fn read(&mut self, into: &mut [u8]) -> Result<(), codec::Error> {
		let len_from_inner = into.len().min(self.0.len());
		into[..len_from_inner].copy_from_slice(&self.0[..len_from_inner]);
		for i in &mut into[len_from_inner..] {
			*i = 0;
		}
		self.0 = &self.0[len_from_inner..];

		Ok(())
	}
}
