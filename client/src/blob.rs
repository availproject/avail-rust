use crate::{
	BlockQueryMode, Client, Error, Options, SubmittableTransaction, TransactionReceipt, platform,
	submission::submitted::WaitOption,
};
use avail_rust_core::{
	DataFormat, H256,
	avail::data_availability::types::BlobTxSummary,
	rpc::{AllowedExtrinsic, blob::BlobInfo, kate::DataProof},
	subxt_core::config::{Hasher, substrate::BlakeTwo256},
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

	pub fn metadata_ext(
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
	/// Returns metadata extrinsic hash
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
	) -> Result<H256, Error> {
		let tx = self.metadata_ext(app_id, blob_hash, blob.len() as u64, commitments, eval_point_seed, eval_claim);
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
	) -> Result<FindBlobTxOutcome, Error> {
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

	/// Returns metadata extrinsic hash
	pub async fn submit(&self, metadata_ext: &[u8], blob: &[u8]) -> Result<H256, Error> {
		let metadata_ext_hash = BlakeTwo256.hash(metadata_ext);
		self.client.chain().blob_submit_blob(metadata_ext, blob).await?;
		Ok(metadata_ext_hash)
	}

	pub async fn submit_and_watch(
		&self,
		metadata_ext: &[u8],
		blob: &[u8],
		opts: impl Into<WaitOption>,
	) -> Result<FindBlobTxOutcome, Error> {
		self.submit_and_watch_inner(metadata_ext, blob, opts.into()).await
	}

	async fn submit_and_watch_inner(
		&self,
		metadata_ext: &[u8],
		blob: &[u8],
		opts: WaitOption,
	) -> Result<FindBlobTxOutcome, Error> {
		let chain_info = self.client.chain().info().await?;
		let metadata_ext_hash = self.submit(metadata_ext, blob).await?;

		let block_height = match opts.mode {
			BlockQueryMode::Finalized => chain_info.finalized_height,
			BlockQueryMode::Best => chain_info.best_height,
		};

		watch_with_timeout(self.client, metadata_ext_hash, block_height, opts).await
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
	) -> Result<FindBlobTxOutcome, Error> {
		let mortality = options.resolve_mortality(self.client).await?;
		let tx = self.metadata_ext(app_id, blob_hash, blob.len() as u64, commitments, eval_point_seed, eval_claim);
		let metadata_ext = tx.sign(signer, options).await?;
		let metadata_ext = metadata_ext.encode();

		opts.max_block_height = opts
			.max_block_height
			.or_else(|| Some(mortality.block_height + mortality.period as u32));

		self.submit_and_watch_inner(&metadata_ext, blob, opts).await
	}
}

#[derive(Debug, Clone)]
pub struct FoundBlobExt {
	pub receipt: TransactionReceipt,
	pub summary: BlobTxSummary,
}

#[derive(Debug, Clone)]
pub enum FindBlobTxOutcome {
	Found(FoundBlobExt),
	NotFound,
	TimedOut,
}

pub async fn watch_with_timeout(
	client: &Client,
	metadata_tx_hash: H256,
	block_height: u32,
	opts: WaitOption,
) -> Result<FindBlobTxOutcome, Error> {
	let future = watch(client, metadata_tx_hash, block_height, opts.mode, opts.max_block_height);
	match platform::timeout(opts.timeout, future).await {
		Ok(result) => result,
		Err(_) => Ok(FindBlobTxOutcome::TimedOut),
	}
}

pub async fn watch(
	client: &Client,
	metadata_tx_hash: H256,
	block_height: u32,
	mode: BlockQueryMode,
	max_block_height: Option<u32>,
) -> Result<FindBlobTxOutcome, Error> {
	let mut sub = client
		.subscribe()
		.blocks()
		.from_height(block_height)
		.mode(mode)
		.build()
		.await?;

	let allow_list = vec![AllowedExtrinsic::from(metadata_tx_hash)];
	loop {
		let block = sub.next().await?;
		if block_height == 0 {
			continue;
		}
		if max_block_height.is_some_and(|x| block.block_height > x) {
			return Ok(FindBlobTxOutcome::NotFound);
		}

		let query = block.value.extrinsics();
		let extrinsics = query
			.rpc(Some(allow_list.clone()), Default::default(), DataFormat::None)
			.await?;

		let Some(extrinsic) = extrinsics.first() else {
			continue;
		};

		let summaries = query.ext_submit_blob_txs_summary().await?.call.blob_txs_summary;
		let Some(summary) = summaries.into_iter().find(|x| x.tx_index == extrinsic.ext_index) else {
			continue;
		};

		let receipt = TransactionReceipt::new(
			client.clone(),
			block.block_hash,
			block.block_height,
			extrinsic.ext_hash,
			extrinsic.ext_index,
		);
		let found = FoundBlobExt { summary, receipt };
		return Ok(FindBlobTxOutcome::Found(found));
	}
}
