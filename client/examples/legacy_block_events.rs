use avail_rust_client::prelude::*;
use avail_rust_core::subxt_core::events::Phase;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	// Empty Block
	let client = Client::new(TURING_ENDPOINT).await?;
	let at = client.chain().block_hash(Some(2923704)).await?.unwrap();

	let encoded_events = client.chain().legacy_block_events(at).await?;
	assert_eq!(encoded_events.len(), 3);
	assert_eq!(
		encoded_events[0].decode_pallet_variant_name(),
		Some((String::from("Treasury"), String::from("UpdatedInactive")))
	);
	assert_eq!(encoded_events[0].decode_phase(), Some(Phase::Initialization));

	assert_eq!(
		encoded_events[1].decode_pallet_variant_name(),
		Some((String::from("System"), String::from("ExtrinsicSuccess")))
	);
	assert_eq!(encoded_events[1].decode_phase(), Some(Phase::ApplyExtrinsic(0)));

	assert_eq!(
		encoded_events[2].decode_pallet_variant_name(),
		Some((String::from("System"), String::from("ExtrinsicSuccess")))
	);
	assert_eq!(encoded_events[2].decode_phase(), Some(Phase::ApplyExtrinsic(1)));

	// Block with Non-default Exts
	let at = client.chain().block_hash(Some(116749)).await?.unwrap();
	let encoded_events = client.chain().legacy_block_events(at).await?;
	assert_eq!(encoded_events.len(), 131);
	assert_eq!(
		encoded_events[6].decode_pallet_variant_name(),
		Some((String::from("System"), String::from("ExtrinsicFailed")))
	);

	Ok(())
}
