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
