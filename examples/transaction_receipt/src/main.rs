use avail::data_availability::{events::DataSubmitted, tx::SubmitData};
use avail_rust_client::{
	block::{BlockExtrinsic, BlockRawExtrinsic, BlockSignedExtrinsic},
	prelude::*,
};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Transaction Creation
	let submittable: SubmittableTransaction = client.tx().data_availability().submit_data(vec![0, 1, 2, 3, 4, 5]);

	// Transaction Submission - Tx Hash + Options used to submit the TX
	let submitted: SubmittedTransaction = submittable.sign_and_submit(&alice(), Options::new(2)).await?;

	// Transaction Receipt - Block Hash, Block Height, Tx Hash, Tx Index
	// This is a handle to a real transaction.
	let receipt: TransactionReceipt = submitted.receipt(false).await?.unwrap();

	// Block State - Included | Finalized | Discarded | DoesNotExist
	let _: BlockState = receipt.block_state().await?;

	// There are 4 ways how you can get relevant data
	// 1. Fetch it as a transaction (signed extrinsic)
	//		- This is your daily driver, covers 99.99 cases
	// 2. Fetch it as a extrinsic (unsigned and signed extrinsic)
	//		- Extrinsic like Timestamp::Set are not signed, so this is a way how to fetch them
	// 3. Fetch it as a raw extrinsic
	// 		- No decoding is done, but metadata like app id, ss58address, nonce and others is available
	// 4. Fetch it as a extrinsic call.
	//		- This is the same as 2., but discards non Extrinsic Call related information.
	{
		// Transaction (Signed Extrinsic) - Extrinsic Signature, Decoded Extrinsic Call, Metadata
		let _: BlockSignedExtrinsic<SubmitData> = receipt.tx().await?;

		// Extrinsic (might be Signed) - [Optional] Extrinsic Signature, Decoded Extrinsic Call, Metadata
		let _: BlockExtrinsic<SubmitData> = receipt.ext().await?;

		// Raw Extrinsic - [Optional] Data, [Optional] SignerPayload, Metadata
		let _: BlockRawExtrinsic = receipt.raw_ext(Default::default()).await?;

		// Decoded Extrinsic Call - In case where you only care about the Extrinsic Call
		let _: SubmitData = receipt.call().await?;
	}

	// Events
	{
		// Extrinsic events
		let events: ExtrinsicEvents = receipt.events().await?;

		// Finding DataSubmitted event
		let _: DataSubmitted = events.first().unwrap();
	}

	// Finding Transaction Receipt from a range of blocks
	{
		let client = Client::new(MAINNET_ENDPOINT).await?;
		let block_start = 1883554;
		let block_end = 1883558;
		let use_best_block = false;
		let tx_hash = "0x5dac43ef6a3c65e4b06d55bc857ffbc3afc59526b4ce36f4bdd86ae0339d893d";

		// Tx Hash can be &str...
		let tr1 = TransactionReceipt::from_range(client.clone(), tx_hash, block_start, block_end, use_best_block)
			.await?
			.unwrap();

		// ...or String...
		let tr2 = TransactionReceipt::from_range(
			client.clone(),
			String::from(tx_hash),
			block_start,
			block_end,
			use_best_block,
		)
		.await?
		.unwrap();

		// ...or H256...
		let tr3 = TransactionReceipt::from_range(
			client.clone(),
			H256::from_str(tx_hash)?,
			block_start,
			block_end,
			use_best_block,
		)
		.await?
		.unwrap();

		// ...They all work the same
		assert!(tr1.block_ref == tr2.block_ref && tr2.block_ref == tr3.block_ref);
		assert!(tr1.tx_ref == tr2.tx_ref && tr2.tx_ref == tr3.tx_ref);
		assert!(tr1.tx_ref.hash == H256::from_str(tx_hash)?);
	}

	Ok(())
}
