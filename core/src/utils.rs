use crate::AccountId;

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
