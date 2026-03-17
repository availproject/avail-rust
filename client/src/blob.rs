use avail_rust_core::{
	H256,
	rpc::{blob::BlobInfo, kate::DataProof},
	subxt_signer::sr25519::Keypair,
};
use codec::Encode;

use crate::{Client, Error, Options, SubmittableTransaction};

pub struct Blob<'a> {
	client: &'a Client,
}

impl<'a> Blob<'a> {
	pub(crate) fn new(client: &'a Client) -> Self {
		Self { client }
	}

	pub async fn get(
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

	pub fn submit_blob_metadata_tx(
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

	pub async fn submit_blob(&self, metadata_signed_transaction: &[u8], blob: &[u8]) -> Result<(), Error> {
		self.client
			.chain()
			.blob_submit_blob(metadata_signed_transaction, blob)
			.await
	}

	#[allow(clippy::too_many_arguments)]
	pub async fn submit_blob_and_blob_metadata(
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
		let tx = self.submit_blob_metadata_tx(
			app_id,
			blob_hash,
			blob.len() as u64,
			commitments,
			eval_point_seed,
			eval_claim,
		);
		let tx_signed = tx.sign(signer, options).await?;

		self.submit_blob(&tx_signed.encode(), blob).await
	}
}
