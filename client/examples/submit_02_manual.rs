use avail_rust_client::prelude::*;
use avail_rust_core::{MultiSignature, SignedPayload, substrate::extrinsic::Preamble};
use codec::Encode;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;
	client.set_retry_policy(RetryPolicy::Disabled);

	let signer = Account::new_from_str("//Charlie")?;
	let account_id = signer.public_key().to_account_id();

	// Manual transaction construction
	//
	// The SDK's .submit() method handles signing, encoding, and submission behind the scenes.
	// This example shows how to do all of that yourself, which is useful when you need full
	// control over the extrinsic bytes — for example, when integrating with an external signer
	// or building transactions offline.
	//
	// The steps are:
	// 1. Fetch nonce and mortality anchor from the chain
	// 2. Build the extension and implicit parameters
	// 3. Construct the raw call bytes
	// 4. Sign, assemble the extrinsic, encode, and submit via RPC

	// Step 1: Fetch the values that the SDK normally resolves for you. The nonce comes from
	// the account state, and the mortality anchor comes from the latest finalized head.
	// If you're building offline, you'd provide these values yourself.
	let nonce = client.finalized().account_nonce(account_id.clone()).await?;
	let chain_info = client.chain().info().await?;

	// Step 2: Build extension (explicit fields like nonce, tip) and implicit parameters
	// (spec version, tx version, genesis hash) that get signed but not included in the
	// extrinsic payload.
	let extension = Extension {
		era: avail_rust_core::Era::mortal(32, chain_info.finalized_height as u64),
		nonce,
		tip: 0,
	};
	let implicit = ExtensionImplicit {
		spec_version: client.online_client().spec_version(),
		tx_version: client.online_client().transaction_version(),
		genesis_hash: client.online_client().genesis_hash(),
		fork_hash: chain_info.finalized_hash,
	};

	// Step 3: Build the raw call. These bytes are [pallet_index, call_index, ...SCALE-encoded args].
	// Here 29=DataAvailability, 1=submit_data, followed by a compact-encoded "hello".
	let call = vec![29u8, 1u8, 8u8, 104u8, 101u8, 108u8, 108u8, 111u8];

	// Step 4: Sign, assemble, encode, submit.
	let signature = SignedPayload::sign_static(&call, &extension, &implicit, &signer);

	let address = MultiAddress::Id(account_id);
	let signature = MultiSignature::Sr25519(signature);
	let preamble = Preamble::Signed(address, signature, extension);
	let extrinsic = Extrinsic::new(preamble, call.into());

	let encoded = extrinsic.encode();
	println!("Encoded: {:?}", const_hex::encode_prefixed(&encoded));

	let ext_hash = client.chain().submit(&encoded).await?;
	println!("Ext Hash: {:?}", ext_hash);

	// Finding the receipt manually
	//
	// Since we bypassed the SDK's submission pipeline, we don't get a SubmittedTransaction handle.
	// Instead we can use TransactionReceipt::from_range() to scan a range of blocks for our
	// transaction by its hash.
	let finalized_height = client.finalized().block_height().await?;
	let receipt = TransactionReceipt::from_range(
		client.clone(),
		ext_hash,
		finalized_height,
		finalized_height + 10,
		BlockQueryMode::Finalized,
	)
	.await?
	.expect("Should be found");

	println!("Included: height={}", receipt.block_height);

	Ok(())
}
