//! This example showcases the following actions:
//! - Creating custom transaction and submitting it
//! - Decoding custom transaction
//!

use avail_rust_client::prelude::*;

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomTransaction {
	pub data: Vec<u8>,
}
impl HasHeader for CustomTransaction {
	const HEADER_INDEX: (u8, u8) = (29u8, 1u8);
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	// Decoding...

	// Decoding Hex Transaction Call to our Custom Transaction
	// For decoding from bytes call `decode_call`
	let ct = CustomTransaction::decode_hex_call("0x1d010c616263").expect("Should work");
	assert_eq!(ct.data, vec![b'a', b'b', b'c']);

	// Decoding whole Hex Transaction to our Custom Transaction
	// For decoding from bytes call `decode_transaction`
	let tx = "0xb90184007e170b74231de8a3b8bbe55e4cda756e1e4eab0807d5637eca2d81d61ac02b15015e7a61c64e171023b165ba4fde6e41bb017a9dab2b357f1fd192c1d2c1f99956cb44df23ff4084b065f31b3b7634e02a081c7f86ca2cbe180b734acd2da3488cd4013c000c1d010c616263";
	let ct = CustomTransaction::decode_hex_transaction(tx).expect("Should work");
	assert_eq!(ct.data, vec![b'a', b'b', b'c']);

	// Decoding whole Hex Transaction to Opaque Transaction and then to Custom Transaction
	// `try_from` accepts &[u8] as well
	let opaq = OpaqueTransaction::try_from(tx).expect("Should work");
	let sig = opaq.signature.as_ref().expect("qed");
	assert_eq!(sig.tx_extra.app_id, 3);
	let ct = CustomTransaction::decode_call(&opaq.call).expect("Should work");
	assert_eq!(ct.data, vec![b'a', b'b', b'c']);

	// Encoding....

	// Just one single line, that's it :)
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let submittable = CustomTransaction { data: vec![0, 1, 2] }.to_submittable(client.clone());

	// Submitting
	let submitted = submittable.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Must be there");
	println!("Block Hash: {:?}", receipt.block_ref.hash);

	Ok(())
}
