use avail_rust_client::prelude::*;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Error> {
	// We enable logging so that we can observe transactions being submitted at the same time
	Client::init_tracing(false).expect("Should work");

	// alice, bob, charlie and dave will do data submission at the same time
	let mut futures: Vec<JoinHandle<Result<(), Error>>> = Vec::new();
	for signer in [alice(), bob(), charlie(), dave()] {
		futures.push(tokio::spawn(async move { task(signer).await }));
	}

	for fut in futures {
		fut.await.expect("Should be successful")?;
	}

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

/*
	Expected Output:

	2025-10-28T14:44:44.168473Z  INFO tx: Submitting Transaction. Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy, Nonce: 1, App Id: 0
	2025-10-28T14:44:44.168474Z  INFO tx: Submitting Transaction. Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, Nonce: 4, App Id: 0
	2025-10-28T14:44:44.169026Z  INFO tx: Transaction Submitted.  Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, Nonce: 4, App Id: 0, Tx Hash: 0xbe7c7f433d67f6211370d1f752616e0c108d37f111771f408339e99992675ae9,
	2025-10-28T14:44:44.169027Z  INFO tx: Transaction Submitted.  Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy, Nonce: 1, App Id: 0, Tx Hash: 0x29133d50d16855e4f4a64fe7264bed100d6ccb591a5acbac728fca5fc7b86788,
	2025-10-28T14:44:44.169270Z  INFO lib: Nonce: 4 Account address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY Current Finalized Height: 16 Mortality End Height: 48
	2025-10-28T14:44:44.169311Z  INFO tx: Submitting Transaction. Address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Nonce: 2, App Id: 0
	2025-10-28T14:44:44.169413Z  INFO lib: Nonce: 1 Account address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy Current Finalized Height: 16 Mortality End Height: 48
	2025-10-28T14:44:44.169646Z  INFO tx: Transaction Submitted.  Address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Nonce: 2, App Id: 0, Tx Hash: 0x6f0329d5ea64cb437026c1c2bda173546acea745c1e3c2a99fdcac67549ece19,
	2025-10-28T14:44:44.169955Z  INFO lib: Nonce: 2 Account address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty Current Finalized Height: 16 Mortality End Height: 48
	2025-10-28T14:44:44.170049Z  INFO lib: Account (4, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (16, 0x2649c7a11def7df72e0b658b7f917cd1ff6faed7f6437c29a25ffaea9d085e7b) found nonce: 4.
	2025-10-28T14:44:44.170388Z  INFO lib: Account (1, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (16, 0x2649c7a11def7df72e0b658b7f917cd1ff6faed7f6437c29a25ffaea9d085e7b) found nonce: 1.
	2025-10-28T14:44:44.170680Z  INFO lib: Account (2, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (16, 0x2649c7a11def7df72e0b658b7f917cd1ff6faed7f6437c29a25ffaea9d085e7b) found nonce: 2.
	2025-10-28T14:44:44.174911Z  INFO tx: Submitting Transaction. Address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y, Nonce: 2, App Id: 0
	2025-10-28T14:44:44.175279Z  INFO tx: Transaction Submitted.  Address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y, Nonce: 2, App Id: 0, Tx Hash: 0xc2e2aae231f66a2f82bdb2b8dd7bd64fca2a3fd570283eadb0e6a430820ab7aa,
	2025-10-28T14:44:44.175521Z  INFO lib: Nonce: 2 Account address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y Current Finalized Height: 16 Mortality End Height: 48
	2025-10-28T14:44:44.176169Z  INFO lib: Account (2, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (16, 0x2649c7a11def7df72e0b658b7f917cd1ff6faed7f6437c29a25ffaea9d085e7b) found nonce: 2.
	2025-10-28T14:44:47.174822Z  INFO lib: Account (4, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (17, 0xcb1d12bfeb0a9b010394ad53ab5cc2ba8599a54a3115209a31faae512379136e) found nonce: 4.
	2025-10-28T14:44:47.175080Z  INFO lib: Account (1, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (17, 0xcb1d12bfeb0a9b010394ad53ab5cc2ba8599a54a3115209a31faae512379136e) found nonce: 1.
	2025-10-28T14:44:47.175307Z  INFO lib: Account (2, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (17, 0xcb1d12bfeb0a9b010394ad53ab5cc2ba8599a54a3115209a31faae512379136e) found nonce: 2.
	2025-10-28T14:44:47.180317Z  INFO lib: Account (2, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (17, 0xcb1d12bfeb0a9b010394ad53ab5cc2ba8599a54a3115209a31faae512379136e) found nonce: 2.
	2025-10-28T14:45:08.198087Z  INFO lib: Account (2, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (18, 0x822744b19f25f3b2451822cdde0faa65b7f986d11c35333522064eb310b70e80) found nonce: 2.
	2025-10-28T14:45:08.198347Z  INFO lib: Account (1, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (18, 0x822744b19f25f3b2451822cdde0faa65b7f986d11c35333522064eb310b70e80) found nonce: 1.
	2025-10-28T14:45:08.198749Z  INFO lib: Account (4, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (18, 0x822744b19f25f3b2451822cdde0faa65b7f986d11c35333522064eb310b70e80) found nonce: 4.
	2025-10-28T14:45:08.200076Z  INFO lib: Account (2, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (18, 0x822744b19f25f3b2451822cdde0faa65b7f986d11c35333522064eb310b70e80) found nonce: 2.
	2025-10-28T14:45:26.216585Z  INFO lib: Account (2, 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty). At block (19, 0xeff1ea021951b760ef9ea55f1189463816fc8717dddc8c33678887eabb431a0c) found nonce: 3. Search is done
	2025-10-28T14:45:26.216585Z  INFO lib: Account (1, 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy). At block (19, 0xeff1ea021951b760ef9ea55f1189463816fc8717dddc8c33678887eabb431a0c) found nonce: 2. Search is done
	2025-10-28T14:45:26.216653Z  INFO lib: Account (2, 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y). At block (19, 0xeff1ea021951b760ef9ea55f1189463816fc8717dddc8c33678887eabb431a0c) found nonce: 3. Search is done
	2025-10-28T14:45:26.216707Z  INFO lib: Account (4, 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY). At block (19, 0xeff1ea021951b760ef9ea55f1189463816fc8717dddc8c33678887eabb431a0c) found nonce: 5. Search is done

*/
