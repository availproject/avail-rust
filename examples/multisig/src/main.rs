use avail_rust_client::{
	avail::{
		multisig::{
			events::{MultisigApproval, MultisigExecuted, NewMultisig},
			types::Timepoint,
		},
		vector::types::Weight,
	},
	avail_rust_core::AccountIdLike,
	prelude::*,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Multisig Signatures
	let (alice, bob, charlie) = (alice(), bob(), charlie());

	// Generating multisig account
	let threshold = 3u16;
	let multisig_account = multi_account_id(&[alice.account_id(), bob.account_id(), charlie.account_id()], threshold);
	println!("Multisig address: {}", multisig_account);
	/*
		Multisig address: 5EAkPWNziBqEnrw6hkjFVu6EJej7Xf9wEK4CXir6YDS4kvUL
	*/

	// Funding multisig account
	fund_multisig_account(&client, &alice, multisig_account).await?;

	// Creating transaction that the multisig account will execute
	let dest = "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy";
	let amount = ONE_AVAIL;
	let submittable = client.tx().balances().transfer_keep_alive(dest, amount);

	// Gathering necessary information to execute multisig calls
	let call_hash = H256::from(submittable.call_hash());
	let call = submittable.call.clone();
	let call_max_weight = submittable.call_info(None).await?.weight;

	/*
	  The first signature creates and approves the multisig transaction. All the next signatures (besides the last one) should
	  use the `nextApproval` function to approve the tx. The last signature should use the `lastApproval` function to approve
	  and execute the multisig tx.

	  In practice it means the following:
	  - If the threshold is 2 do the following:
		- firstApproval
		- lastApproval
	  - If the threshold is 4 do the following:
		- firstApproval
		- nextApproval
		- nextApproval
		- lastApproval
	*/

	// First Approval
	let other_signatures = vec![bob.account_id(), charlie.account_id()];
	let receipt = first_approval(&client, &alice, threshold, other_signatures, call_hash, call_max_weight).await?;

	// Approve existing Multisig Transaction
	let timepoint = Timepoint {
		height: receipt.block_ref.height,
		index: receipt.tx_ref.index,
	};
	let other_signatures = vec![alice.account_id(), charlie.account_id()];
	next_approval(&client, &bob, threshold, other_signatures, timepoint, call_hash, call_max_weight).await?;

	// Execute Multisig
	let other_signatures = vec![alice.account_id(), bob.account_id()];
	last_approval(&client, &charlie, threshold, other_signatures, timepoint, call, call_max_weight).await?;

	Ok(())
}

async fn fund_multisig_account(client: &Client, signer: &Keypair, multisig_account: AccountId) -> Result<(), Error> {
	let amount = TEN_AVAIL;
	let submittable = client.tx().balances().transfer_keep_alive(multisig_account, amount);
	let submitted = submittable.sign_and_submit(signer, Default::default()).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert!(events.is_extrinsic_success_present());

	Ok(())
}

async fn first_approval(
	client: &Client,
	signer: &Keypair,
	threshold: u16,
	other_signatures: Vec<impl Into<AccountIdLike>>,
	call_hash: H256,
	max_weight: Weight,
) -> Result<TransactionReceipt, Error> {
	let submittable = client
		.tx()
		.multisig()
		.approve_as_multi(threshold, other_signatures, None, call_hash, max_weight);

	let submitted = submittable.sign_and_submit(signer, Default::default()).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be included in a block");
	let events = receipt.events().await?;
	assert!(events.is_extrinsic_success_present());

	let event = events.first::<NewMultisig>().expect("Should be there");
	println!("Approving: {}, Call Hash: {:?}, Multisig: {}", event.approving, event.call_hash, event.multisig);
	/*
		Approving: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,
		Call Hash: 0x543b0d9d49971c569ca8f66190f80a01442f38b18ab062a2cb18025e3f3ec332,
		Multisig: 5EAkPWNziBqEnrw6hkjFVu6EJej7Xf9wEK4CXir6YDS4kvUL
	*/

	Ok(receipt)
}

async fn next_approval(
	client: &Client,
	signer: &Keypair,
	threshold: u16,
	other_signatures: Vec<impl Into<AccountIdLike>>,
	timepoint: Timepoint,
	call_hash: H256,
	max_weight: Weight,
) -> Result<(), Error> {
	let submittable =
		client
			.tx()
			.multisig()
			.approve_as_multi(threshold, other_signatures, Some(timepoint), call_hash, max_weight);

	let submitted = submittable.sign_and_submit(signer, Default::default()).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be included in a block");
	let events = receipt.events().await?;
	assert!(events.is_extrinsic_success_present());

	let event = events.first::<MultisigApproval>().expect("Should be there");
	let (approving, call_hash, multisig) = (event.approving, event.call_hash, event.multisig);
	println!(
		"Approving: {}, Call Hash: {:?}, Multisig: {}, Timepoint: {:?}",
		approving, call_hash, multisig, event.timepoint
	);
	/*
		Approving: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty,
		Call Hash: 0x543b0d9d49971c569ca8f66190f80a01442f38b18ab062a2cb18025e3f3ec332,
		Multisig: 5EAkPWNziBqEnrw6hkjFVu6EJej7Xf9wEK4CXir6YDS4kvUL,
		Timepoint: Timepoint { height: 3, index: 1 }
	*/

	Ok(())
}

async fn last_approval(
	client: &Client,
	signer: &Keypair,
	threshold: u16,
	other_signatures: Vec<impl Into<AccountIdLike>>,
	timepoint: Timepoint,
	call: ExtrinsicCall,
	max_weight: Weight,
) -> Result<(), Error> {
	let submittable = client
		.tx()
		.multisig()
		.as_multi(threshold, other_signatures, Some(timepoint), call, max_weight);

	let submitted = submittable.sign_and_submit(signer, Default::default()).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be included in a block");
	let events = receipt.events().await?;
	assert!(events.is_extrinsic_success_present());
	assert!(events.multisig_executed_successfully().expect("Event should be there"));

	let event = events.first::<MultisigExecuted>().expect("Should be there");
	assert!(event.result.is_ok());

	let (approving, call_hash, multisig) = (event.approving, event.call_hash, event.multisig);
	println!(
		"Approving: {}, Call Hash: {:?}, Multisig: {}, Timepoint: {:?}, Result: {:?}",
		approving, call_hash, multisig, event.timepoint, event.result
	);
	/*
		Approving: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y,
		Call Hash: 0x543b0d9d49971c569ca8f66190f80a01442f38b18ab062a2cb18025e3f3ec332,
		Multisig: 5EAkPWNziBqEnrw6hkjFVu6EJej7Xf9wEK4CXir6YDS4kvUL,
		Timepoint: Timepoint { height: 3, index: 1 }, Result: Ok(())
	*/

	Ok(())
}
