use avail_rust::{
	avail::{self, runtime_types::bounded_collections::bounded_vec::BoundedVec},
	error::ClientError,
	rpc, utils, Cell, Options, SDK,
};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// author_rotate_keys
	let value = rpc::author::rotate_keys(&sdk.rpc_client).await?;
	let value = utils::deconstruct_session_keys(value)?;
	dbg!(value);
	/*	Output
	SessionKeys {
		babe: Public(...),
		grandpa: Public(...),
		im_online: Public(...),
		authority_discovery: Public(...),
	}
	*/

	// chain_get_block
	let value = rpc::chain::get_block(&sdk.rpc_client, None).await?;
	dbg!(value);
	/*	Output
	BlockDetails {
		block: Block {
			header: AvailHeader {
				parent_hash: 0x4753c70a0652f50ee24f19ea402c1377ce5ab08fc5e0f801123e8116e5e1fcf8,
				number: 495,
				state_root: 0x22470c3402bee3cd95c10b9303e61019aaec0603cbfc197eca646c94ba9332f1,
				extrinsics_root: 0x609ed0e14f3252c9f59ab59004ea458d7927a5bd81f241651634266b7098f415,
				digest: Digest {...},
				extension: V3(
					HeaderExtension {
						app_lookup: CompactDataLookup {
							size: 0,
							index: [],
						},
						commitment: KateCommitment {
							rows: 0,
							cols: 0,
							commitment: [],
							data_root: 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5,
						},
					},
				),
			},
			extrinsics: [...],
		},
		justifications: None,
	}
	*/

	// chain_get_block_hash
	let value = rpc::chain::get_block_hash(&sdk.rpc_client, None).await?;
	dbg!(value);
	/*	Output
		0xc4e0a9a2ef80ddc1d70c9946d8a6f86ca4b15053b39ba56709222f01ddc64561
	*/

	// chain_get_finalized_head
	let value = rpc::chain::get_finalized_head(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		0x2c896c9faae4e111f1fbeb955be5e999a328846969b59a7a7c64eadc4701122a
	*/

	// chain_get_header
	let value = rpc::chain::get_header(&sdk.rpc_client, None).await?;
	dbg!(value);
	/*	Output
	AvailHeader {
		parent_hash: 0x4753c70a0652f50ee24f19ea402c1377ce5ab08fc5e0f801123e8116e5e1fcf8,
		number: 495,
		state_root: 0x22470c3402bee3cd95c10b9303e61019aaec0603cbfc197eca646c94ba9332f1,
		extrinsics_root: 0x609ed0e14f3252c9f59ab59004ea458d7927a5bd81f241651634266b7098f415,
		digest: Digest {...},
		extension: V3(
			HeaderExtension {
				app_lookup: CompactDataLookup {
					size: 0,
					index: [],
				},
				commitment: KateCommitment {
					rows: 0,
					cols: 0,
					commitment: [],
					data_root: 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5,
				},
			},
		),
	}
	*/

	// system_account_next_index
	let account = String::from("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
	let value = rpc::system::account_next_index(&sdk.rpc_client, account).await?;
	dbg!(value);
	/*	Output
		2
	*/

	// system_chain
	let value = rpc::system::chain(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		"Avail Development Network"
	*/

	// system_chain_type
	let value = rpc::system::chain_type(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		"Development"
	*/

	// system_health
	let value = rpc::system::health(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
	SystemHealth {
		peers: 0,
		is_syncing: false,
		should_have_peers: false,
	}
	*/

	// system_local_listen_addresses
	let value = rpc::system::local_listen_addresses(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
	value = [
		"/ip6/fe81::a234:6e32:1034:3c3b/tcp/30333/p2p/12D3KooWRajsCfp1NR15iN7PcwcFAG3LB7iGDKUBosHkevNRQLYs",
		"/ip4/192.168.1.103/tcp/30333/p2p/12D3KooWRajsCfp1NR15iN7PcwcFAG3LB7iGDKUBosHkevNRQLYs",
		"/ip6/::1/tcp/30333/p2p/12D3KooWRajsCfp1NR15iN7PcwcFAG3LB7iGDKUBosHkevNRQLYs",
		"/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRajsCfp1NR15iN7PcwcFAG3LB7iGDKUBosHkevNRQLYs",
	]
	*/

	// system_local_peer_id
	let value = rpc::system::local_peer_id(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		"12D3KooWRajsCfp1NR15iN7PcwcFAG3LB7iGDKUBosHkevNRQLYs"
	*/

	// system_name
	let value = rpc::system::name(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		"Avail Node"
	*/

	// system_node_roles
	let value = rpc::system::node_roles(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
	[
		Authority,
	]
	*/

	// system_peers
	let value = rpc::system::peers(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		[]
	*/

	// system_properties
	let value = rpc::system::properties(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
	{
		"ss58Format": Number(42),
		"tokenDecimals": Number(18),
		"tokenSymbol": String("AVAIL"),
	}
	*/

	// system_system_sync_state
	let value = rpc::system::sync_state(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
	SyncState {
		starting_block: 0,
		current_block: 495,
		highest_block: 495,
	}
	*/

	// system_version
	let value = rpc::system::version(&sdk.rpc_client).await?;
	dbg!(value);
	/*	Output
		"2.2.1-55da578d34b"
	*/

	// TransactionPaymentApi_query_info
	let payload = avail::tx()
		.data_availability()
		.submit_data(BoundedVec(vec![1]));
	let keypair = SDK::alice()?;
	let account = keypair.public_key().to_account_id();

	let options = Options::new().app_id(1);
	let populated_options = options
		.build(&sdk.online_client, &sdk.rpc_client, &account)
		.await?;

	let params = populated_options.build(&sdk.rpc_client).await?;
	let tx = sdk
		.online_client
		.tx()
		.create_signed(&payload, &keypair, params)
		.await?;
	let partial_fee_estimate = tx.partial_fee_estimate().await?;
	dbg!(partial_fee_estimate);
	/*	Output
		124684322202721409
	*/

	// TransactionPaymentApi_query_fee_details
	let len_bytes: [u8; 4] = (tx.encoded().len() as u32).to_le_bytes();
	let encoded_with_len = [tx.encoded(), &len_bytes[..]].concat();

	let fee_details =
		rpc::payment::query_fee_details(&sdk.rpc_client, encoded_with_len.into(), None).await?;
	dbg!(fee_details);
	/*	Output
	FeeDetails {
		inclusion_fee: Some(
			InclusionFee {
				base_fee: 124414000000000000,
				len_fee: 11400000000000,
				adjusted_weight_fee: 259321813738397,
			},
		),
		tip: 0,
	}
	*/

	// kate_block_length
	let value = rpc::kate::block_length(&sdk.rpc_client, None).await?;
	dbg!(value);
	/*	Output
	BlockLength {
		max: PerDispatchClass {
			normal: 2097152,
			operational: 2097152,
			mandatory: 2097152,
		},
		cols: BlockLengthColumns(
			256,
		),
		rows: BlockLengthRows(
			256,
		),
		chunk_size: 32,
	}
	*/

	// kate_query_data_proof
	let data = String::from("My Data").into_bytes();
	let tx = sdk.tx.data_availability.submit_data(data);
	let result = tx.execute_and_watch_finalization(&keypair, None).await?;
	let (tx_index, block_hash) = (result.tx_index, Some(result.block_hash));
	let value = rpc::kate::query_data_proof(&sdk.rpc_client, tx_index, block_hash).await?;
	dbg!(value);
	/*	Output
	ProofResponse {
		data_proof: DataProof {
			roots: TxDataRoots {
				data_root: 0xd6e516bbf0b0d964a6a6a41a18c58a2eac4757001c2338a8601c4cc961332fda,
				blob_root: 0x29c73490baca9fe2b11095a69294de4b4a86bcb3a2eb3cd04b51dfdd0b4030f9,
				bridge_root: 0x0000000000000000000000000000000000000000000000000000000000000000,
			},
			proof: [],
			number_of_leaves: 1,
			leaf_index: 0,
			leaf: 0x47a59a7805e0bfe350ee0395d426c15770edc03fee72aa6532b5bbcffaf28030,
		},
		message: None,
	}
	*/

	// kate_query_proof
	let cells = vec![Cell::from((0u32, 0u32))];
	let value = rpc::kate::query_proof(&sdk.rpc_client, cells, block_hash).await?;
	dbg!(value);
	/*	Output
	[
		(
			2178534751726990040338027377623275511556638494274780568875624948149315822336,
			GProof(
				[...],
			),
		),
	]
	*/

	// kate_query_rows
	let rows = vec![0u32];
	let value = rpc::kate::query_rows(&sdk.rpc_client, rows, block_hash).await?;
	dbg!(value);
	/*	Output
	[
		[
			2178534751726990040338027377623275511556638494274780568875624948149315822336,
			69809044805081050561201039752112594468796256047454289799440609083602104564736,
			26941852917393734161602180963833199552029986735939578666038548832600818441216,
			14351520191331507525755130937317610561547699892218140156652644610507664261120,
		],
	]
	*/

	Ok(())
}
