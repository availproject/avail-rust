use avail_rust_client::prelude::*;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Error> {
	// We enable logging so that we can observe transactions being submitted at the same time
	Client::toggle_tracing(true, false);

	// alice, bob, charlie and dave will do data submission at the same time
	let mut futures: Vec<JoinHandle<Result<(), Error>>> = Vec::new();
	for signer in [alice(), bob(), charlie(), dave()] {
		futures.push(tokio::spawn(async move { task(signer).await }));
	}

	for fut in futures {
		fut.await.expect("Should be successful")?;
	}
	/*
		2025-09-20T23:24:26.508037Z  INFO tx: Submitting Transaction. Address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Nonce: 0, App Id: 0
		2025-09-20T23:24:26.508236Z  INFO tx: Submitting Transaction. Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy, Nonce: 0, App Id: 0
		2025-09-20T23:24:26.508579Z  INFO tx: Transaction Submitted.  Address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Nonce: 0, App Id: 0, Tx Hash: 0x585397598fdc510377a0f2a9a6df146eba7316afb3226d4dbed73b97f0b076c0,
		2025-09-20T23:24:26.508590Z  INFO tx: Submitting Transaction. Address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y, Nonce: 0, App Id: 0
		2025-09-20T23:24:26.508719Z  INFO tx: Transaction Submitted.  Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy, Nonce: 0, App Id: 0, Tx Hash: 0x8eeee60f35108cc20b72864a2a952ebe9d2116c95b502ab6e878eea39fa26f7c,
		2025-09-20T23:24:26.508854Z  INFO lib: Nonce: 0 Account address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty Current Finalized Height: 228 Mortality End Height: 260
		2025-09-20T23:24:26.508975Z  INFO lib: Nonce: 0 Account address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy Current Finalized Height: 228 Mortality End Height: 260
		2025-09-20T23:24:26.509044Z  INFO tx: Transaction Submitted.  Address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y, Nonce: 0, App Id: 0, Tx Hash: 0x96309e1abee46105f662da46d18fdfde3bc5e1262fe5054f722aa489a3a954fe,
		2025-09-20T23:24:26.509251Z  INFO lib: Nonce: 0 Account address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y Current Finalized Height: 228 Mortality End Height: 260
		2025-09-20T23:24:26.509835Z  INFO lib: Account (0, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (228, 0x1326e083323923419c2dc6416a972c714f7fea5b53811b7b51a4436fe4a06df3) found nonce: 0.
		2025-09-20T23:24:26.509877Z  INFO lib: Account (0, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (228, 0x1326e083323923419c2dc6416a972c714f7fea5b53811b7b51a4436fe4a06df3) found nonce: 0.
		2025-09-20T23:24:26.510128Z  INFO lib: Account (0, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (228, 0x1326e083323923419c2dc6416a972c714f7fea5b53811b7b51a4436fe4a06df3) found nonce: 0.
		2025-09-20T23:24:26.511924Z  INFO tx: Submitting Transaction. Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, Nonce: 3, App Id: 0
		2025-09-20T23:24:26.512297Z  INFO tx: Transaction Submitted.  Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, Nonce: 3, App Id: 0, Tx Hash: 0x07a569323a19b22a88a001ce6254b19d94f48abea7ae5f08fb116f2c2f72db02,
		2025-09-20T23:24:26.512500Z  INFO lib: Nonce: 3 Account address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY Current Finalized Height: 228 Mortality End Height: 260
		2025-09-20T23:24:26.513128Z  INFO lib: Account (3, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (228, 0x1326e083323923419c2dc6416a972c714f7fea5b53811b7b51a4436fe4a06df3) found nonce: 3.
		2025-09-20T23:24:35.514837Z  INFO lib: Account (0, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (229, 0x790c5366d1213e7adfae1bd476adc21a231807824ad5778d25b5a04e8bcfd25e) found nonce: 0.
		2025-09-20T23:24:35.515466Z  INFO lib: Account (0, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (229, 0x790c5366d1213e7adfae1bd476adc21a231807824ad5778d25b5a04e8bcfd25e) found nonce: 0.
		2025-09-20T23:24:35.515599Z  INFO lib: Account (0, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (229, 0x790c5366d1213e7adfae1bd476adc21a231807824ad5778d25b5a04e8bcfd25e) found nonce: 0.
		2025-09-20T23:24:35.517437Z  INFO lib: Account (3, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (229, 0x790c5366d1213e7adfae1bd476adc21a231807824ad5778d25b5a04e8bcfd25e) found nonce: 3.
		2025-09-20T23:24:38.517956Z  INFO lib: Account (0, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (230, 0xdd876d66ff622c7bf68844a5afc3ad1de7020cb18b488083956920cb508d2af1) found nonce: 0.
		2025-09-20T23:24:38.518193Z  INFO lib: Account (0, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (230, 0xdd876d66ff622c7bf68844a5afc3ad1de7020cb18b488083956920cb508d2af1) found nonce: 0.
		2025-09-20T23:24:38.518269Z  INFO lib: Account (0, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (230, 0xdd876d66ff622c7bf68844a5afc3ad1de7020cb18b488083956920cb508d2af1) found nonce: 0.
		2025-09-20T23:24:38.518829Z  INFO lib: Account (3, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (230, 0xdd876d66ff622c7bf68844a5afc3ad1de7020cb18b488083956920cb508d2af1) found nonce: 3.
		2025-09-20T23:24:47.524868Z  INFO lib: Account (0, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (231, 0xe60cf4dddf74f61a2b4e7fa57fc0f26288d9a52fa80e1252513fc38b8567348a) found nonce: 1. Search is done
		2025-09-20T23:24:47.524868Z  INFO lib: Account (0, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (231, 0xe60cf4dddf74f61a2b4e7fa57fc0f26288d9a52fa80e1252513fc38b8567348a) found nonce: 1. Search is done
		2025-09-20T23:24:47.525024Z  INFO lib: Account (0, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (231, 0xe60cf4dddf74f61a2b4e7fa57fc0f26288d9a52fa80e1252513fc38b8567348a) found nonce: 1. Search is done
		2025-09-20T23:24:47.525030Z  INFO lib: Account (3, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (231, 0xe60cf4dddf74f61a2b4e7fa57fc0f26288d9a52fa80e1252513fc38b8567348a) found nonce: 4. Search is done
	*/

	Ok(())
}

async fn task(account: Keypair) -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Transaction Submission
	let tx = client.tx().data_availability().submit_data("It works");
	let st = tx.sign_and_submit(&account, Options::default()).await?;

	// Fetching Transaction Receipt
	st.receipt(false).await?.expect("Should be there");

	Ok(())
}
