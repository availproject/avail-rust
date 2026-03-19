use crate::{
	BlockQueryMode, Client, Error, Options, SubmittableTransaction, platform, submission::submitted::WaitOption,
};
use avail_rust_core::{
	H256,
	avail::data_availability::types::BlobTxSummary,
	rpc::{blob::BlobInfo, kate::DataProof},
	subxt_signer::sr25519::Keypair,
};
use codec::Encode;

pub struct Blob<'a> {
	client: &'a Client,
}

impl<'a> Blob<'a> {
	pub(crate) fn new(client: &'a Client) -> Self {
		Self { client }
	}

	pub async fn fetch(
		&self,
		blob_hash: H256,
		block_hash: Option<H256>,
	) -> Result<avail_rust_core::rpc::blob::Blob, Error> {
		self.client.chain().blob_get_blob(blob_hash, block_hash).await
	}

	/// Retrieve indexed blob info
	pub async fn info(&self, blob_hash: H256) -> Result<BlobInfo, Error> {
		self.client.chain().blob_get_blob_info(blob_hash).await
	}

	/// Return inclusion proof for a blob. If `at` is `Some(hash)` the proof is computed for that block,
	/// otherwise the node will try to use its indexed finalized block for the blob.
	pub async fn inclusion_proof(&self, blob_hash: H256, at: Option<H256>) -> Result<DataProof, Error> {
		self.client.chain().blob_inclusion_proof(blob_hash, at).await
	}

	pub fn metadata_tx(
		&self,
		app_id: u32,
		blob_hash: H256,
		size: u64,
		commitments: Vec<u8>,
		eval_point_seed: Option<[u8; 32]>,
		eval_claim: Option<[u8; 16]>,
	) -> SubmittableTransaction {
		self.client
			.tx()
			.data_availability()
			.submit_blob_metadata(app_id, blob_hash, size, commitments, eval_point_seed, eval_claim)
	}

	#[allow(clippy::too_many_arguments)]
	pub async fn submit_with_metadata(
		&self,
		app_id: u32,
		blob: &[u8],
		blob_hash: H256,
		commitments: Vec<u8>,
		eval_point_seed: Option<[u8; 32]>,
		eval_claim: Option<[u8; 16]>,
		signer: &Keypair,
		options: Options,
	) -> Result<(), Error> {
		let tx = self.metadata_tx(app_id, blob_hash, blob.len() as u64, commitments, eval_point_seed, eval_claim);
		let tx_signed = tx.sign(signer, options).await?;

		self.submit(&tx_signed.encode(), blob).await
	}

	#[allow(clippy::too_many_arguments)]
	pub async fn submit_with_metadata_and_watch(
		&self,
		app_id: u32,
		blob: &[u8],
		blob_hash: H256,
		commitments: Vec<u8>,
		eval_point_seed: Option<[u8; 32]>,
		eval_claim: Option<[u8; 16]>,
		signer: &Keypair,
		options: Options,
		opts: impl Into<WaitOption>,
	) -> Result<FindBlobTxSummaryOutcome, Error> {
		Self::submit_with_metadata_and_watch_inner(
			&self,
			app_id,
			blob,
			blob_hash,
			commitments,
			eval_point_seed,
			eval_claim,
			signer,
			options,
			opts.into(),
		)
		.await
	}

	pub async fn submit(&self, metadata: &[u8], blob: &[u8]) -> Result<(), Error> {
		self.client.chain().blob_submit_blob(metadata, blob).await
	}

	pub async fn submit_and_watch(
		&self,
		metadata: &[u8],
		blob: &[u8],
		blob_hash: H256,
		opts: impl Into<WaitOption>,
	) -> Result<FindBlobTxSummaryOutcome, Error> {
		self.submit_and_watch_inner(metadata, blob, blob_hash, opts.into())
			.await
	}

	async fn submit_and_watch_inner(
		&self,
		metadata_signed_transaction: &[u8],
		blob: &[u8],
		blob_hash: H256,
		opts: WaitOption,
	) -> Result<FindBlobTxSummaryOutcome, Error> {
		let chain_info = self.client.chain().info().await?;
		self.submit(metadata_signed_transaction, blob).await?;

		let block_height = match opts.mode {
			BlockQueryMode::Finalized => chain_info.finalized_height,
			BlockQueryMode::Best => chain_info.best_height,
		};

		watch_with_timeout(self.client, blob_hash, block_height, opts).await
	}

	#[allow(clippy::too_many_arguments)]
	async fn submit_with_metadata_and_watch_inner(
		&self,
		app_id: u32,
		blob: &[u8],
		blob_hash: H256,
		commitments: Vec<u8>,
		eval_point_seed: Option<[u8; 32]>,
		eval_claim: Option<[u8; 16]>,
		signer: &Keypair,
		options: Options,
		mut opts: WaitOption,
	) -> Result<FindBlobTxSummaryOutcome, Error> {
		let mortality = options.resolve_mortality(self.client).await?;
		let tx = self.metadata_tx(app_id, blob_hash, blob.len() as u64, commitments, eval_point_seed, eval_claim);
		let tx_signed = tx.sign(signer, options).await?;

		opts.max_block_height = opts
			.max_block_height
			.or_else(|| Some(mortality.block_height + mortality.period as u32));

		self.submit_and_watch_inner(&tx_signed.encode(), blob, blob_hash, opts)
			.await
	}
}

#[derive(Debug, Clone)]
pub struct FoundBlobInformation {
	pub summary: BlobTxSummary,
	pub block_height: u32,
	pub block_hash: H256,
}

#[derive(Debug, Clone)]
pub enum FindBlobTxSummaryOutcome {
	Found(FoundBlobInformation),
	NotFound,
	TimedOut,
}

pub async fn watch_with_timeout(
	client: &Client,
	blob_hash: H256,
	block_height: u32,
	opts: WaitOption,
) -> Result<FindBlobTxSummaryOutcome, Error> {
	let future = watch(client, blob_hash, block_height, opts.mode, opts.max_block_height);
	match platform::timeout(opts.timeout, future).await {
		Ok(result) => result,
		Err(_) => Ok(FindBlobTxSummaryOutcome::TimedOut),
	}
}

pub async fn watch(
	client: &Client,
	blob_hash: H256,
	block_height: u32,
	mode: BlockQueryMode,
	max_block_height: Option<u32>,
) -> Result<FindBlobTxSummaryOutcome, Error> {
	let mut sub = client
		.subscribe()
		.blocks()
		.from_height(block_height)
		.mode(mode)
		.build()
		.await?;

	loop {
		let block = sub.next().await?;
		if block_height == 0 {
			continue;
		}
		if max_block_height.is_some_and(|x| block.block_height > x) {
			return Ok(FindBlobTxSummaryOutcome::NotFound);
		}

		let query = block.value.extrinsics();
		let ext = query.ext_submit_blob_txs_summary().await?;
		let Some(summary) = ext.call.blob_txs_summary.into_iter().find(|x| x.hash == blob_hash) else {
			continue;
		};

		let found = FoundBlobInformation { summary, block_height, block_hash: block.block_hash };
		return Ok(FindBlobTxSummaryOutcome::Found(found));
	}
}
