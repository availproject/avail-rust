use avail_rust_client::{prelude::*, subxt_core::config::Hasher};
use avail_rust_core::{BlakeTwo256, ExtrinsicPayload, MultiSignature};
use codec::Encode;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	// Shortcut
	let seed = "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Charlie";
	let my_signer = Keypair::from_str(seed).expect("Should work");

	// Creating a extrinsic from scratch
	let client = Client::new(TURING_ENDPOINT).await?;

	// call data needs to be scale encoded
	let call_data = vec![66u8, 66u8, 67u8, 67u8].encode();
	let call = avail_rust_core::ExtrinsicCall::new(29, 1, call_data);

	// We use the client to build the payload. We could do it manually but this is easier.
	let account_id = my_signer.account_id();
	let payload = client.chain().build_payload(&account_id, &call, Options::new()).await?;
	let signature = sign(&my_signer, &payload);

	let address = MultiAddress::Id(account_id);
	let signature = MultiSignature::Sr25519(signature);
	let signature = Some(ExtrinsicSignature { address, signature, extra: payload.extra.clone() });
	let extrinsic = avail_rust_core::GenericExtrinsic { call: payload.call, signature };

	// We need finalized height in order to find the tx receipt
	let finalized_height = client.finalized().block_height().await?;

	// Extrinsic needs to be scale encoded
	let extrinsic = extrinsic.encode();
	let ext_hash = client.chain().submit_raw(&extrinsic).await?;
	println!("Ext Hash: {:?}", ext_hash);

	// Finding tx receipt
	let receipt =
		TransactionReceipt::from_range(client.clone(), ext_hash, finalized_height, finalized_height + 10, false)
			.await?
			.expect("Should be found");
	println!(
		"Block Height: {}, Block Hash: {:?}, Ext Hash: {:?}, Ext Index: {}",
		receipt.block_height, receipt.block_hash, receipt.ext_hash, receipt.ext_index
	);

	Ok(())
}

pub fn sign(signer: &Keypair, payload: &ExtrinsicPayload) -> [u8; 64] {
	let call = payload.call.as_ref();
	let size_hint = call.size_hint() + payload.extra.size_hint() + payload.additional.size_hint();

	let mut data: Vec<u8> = Vec::with_capacity(size_hint);
	payload.call.encode_to(&mut data);
	payload.extra.encode_to(&mut data);
	payload.additional.encode_to(&mut data);

	if data.len() > 256 {
		let hash = BlakeTwo256::hash(&data);
		signer.sign(hash.as_ref()).0
	} else {
		signer.sign(&data).0
	}
}

/*
	Expected Output:

	Ext Hash: 0xb129caec0d738de2770fb64d50f68bc624bd827bda2c21c110ff4b2105b48345
	Block Height: 2503855, Block Hash: 0x6723323b1898174c0948d4745519640082b4285b5052469417680774210f6f73, Ext Hash: 0xb129caec0d738de2770fb64d50f68bc624bd827bda2c21c110ff4b2105b48345, Ext Index: 1
*/
