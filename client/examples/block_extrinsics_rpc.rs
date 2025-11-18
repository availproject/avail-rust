use avail_rust_client::prelude::*;
use avail_rust_core::{
	decoded_extrinsics::SignedExtrinsic,
	rpc::{ExtrinsicInfo, system::fetch_extrinsics::Options},
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);

	let opts = Options::new().encode_as(EncodeSelector::None);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159: {}", infos.len());
	printout_details(infos);
	println!("");

	// 1
	let opts = Options::new().encode_as(EncodeSelector::None).filter(1u32);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with Extrinsic Index 1: {}", infos.len());
	printout_details(infos);
	println!("");

	let ext_hash = H256::from_str("0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5").unwrap();
	let opts = Options::new().encode_as(EncodeSelector::None).filter(ext_hash);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with Extrinsic Hash {:?}: {}", ext_hash, infos.len());
	printout_details(infos);
	println!("");

	// 2
	let opts = Options::new().encode_as(EncodeSelector::None).filter(vec![1u32, 3u32]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with tx index filter: {}", infos.len());
	printout_details(infos);
	println!("");

	let ext_hash_1 = H256::from_str("0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5").unwrap();
	let ext_hash_2 = H256::from_str("0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218").unwrap();
	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.filter(vec![ext_hash_1, ext_hash_2]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with tx hash filter: {}", infos.len());
	printout_details(infos);
	println!("");

	// 3
	let opts = Options::new().encode_as(EncodeSelector::None).filter(29u8);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with pallet id 29: {}", infos.len());
	printout_details(infos);
	println!("");

	let opts = Options::new().encode_as(EncodeSelector::None).filter((29u8, 100u8));
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with pallet id 29 and variant id 100: {}", infos.len());
	printout_details(infos);
	println!("");

	// 4
	let opts = Options::new().encode_as(EncodeSelector::None).filter(vec![3u8, 39u8]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with pallet id 3 and 39: {}", infos.len());
	printout_details(infos);
	println!("");

	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.filter(vec![(29u8, 100u8), (3u8, 0u8)]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with PV 29/100 or 3/0: {}", infos.len());
	printout_details(infos);
	println!("");

	// 5
	let opts = Options::new().encode_as(EncodeSelector::None).app_id(246);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with app id 246: {}", infos.len());
	printout_details(infos);
	println!("");

	let opts = Options::new().encode_as(EncodeSelector::None).nonce(2221);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with nonce 2221: {}", infos.len());
	printout_details(infos);
	println!("");

	let address = "5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA";
	let opts = Options::new().encode_as(EncodeSelector::None).ss58_address(address);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with ss58 address {}: {}", address, infos.len());
	printout_details(infos);
	println!("");

	// 6
	let address = "5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA";
	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.app_id(246)
		.ss58_address(address);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with app id 246 and address {}: {}", address, infos.len());
	printout_details(infos);
	println!("");

	// 7
	let address = "5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA";
	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.filter((29, 1))
		.app_id(246)
		.ss58_address(address);
	let infos = block.extrinsic_infos(opts).await?;
	println!(
		"Number of data submission extrinsics in block 2470159 with app id 246 and address {}: {}",
		address,
		infos.len()
	);
	printout_details(infos);
	println!("");

	// 8
	let opts = Options::new().encode_as(EncodeSelector::Call).filter(vec![0u32, 1u32]);
	let mut infos = block.extrinsic_infos(opts).await?;
	printout_details(infos.clone());

	let data = infos[0].data.take().unwrap();
	let call = avail::timestamp::tx::Set::from_call(data).unwrap();
	println!("Timestamp::Set now: {}", call.now);

	let data = infos[1].data.take().unwrap();
	let call = avail::data_availability::tx::SubmitData::from_call(data).unwrap();
	println!("DataAvailability::SubmitData data: {}", String::from_utf8(call.data).unwrap());
	println!("");

	// 9
	let opts = Options::new()
		.encode_as(EncodeSelector::Extrinsic)
		.filter(vec![0u32, 1u32]);
	let mut infos = block.extrinsic_infos(opts).await?;
	printout_details(infos.clone());

	let data = infos[0].data.take().unwrap();
	let extrinsic = EncodedExtrinsic::try_from(&data).unwrap();
	println!(
		"Encoded Extrinsic Timestamp::Set call length: {}, Tip: {:?}",
		extrinsic.call.len(),
		extrinsic.signature.map(|x| x.extra.tip)
	);

	let extrinsic = Extrinsic::<avail::timestamp::tx::Set>::try_from(&data).unwrap();
	println!(
		"Extrinsic Timestamp::Set now: {}, Tip: {:?}",
		extrinsic.call.now,
		extrinsic.signature.map(|x| x.extra.tip)
	);

	let data = infos[1].data.take().unwrap();
	let extrinsic = EncodedExtrinsic::try_from(&data).unwrap();
	println!(
		"Encoded Extrinsic DataAvailability::SubmitData call length: {}, Tip: {:?}",
		extrinsic.call.len(),
		extrinsic.signature.map(|x| x.extra.tip)
	);

	let extrinsic = Extrinsic::<avail::data_availability::tx::SubmitData>::try_from(&data).unwrap();
	println!(
		"Extrinsic DataAvailability::SubmitData data: {}, Tip: {:?}",
		String::from_utf8(extrinsic.call.data).unwrap(),
		extrinsic.signature.map(|x| x.extra.tip)
	);

	let extrinsic = SignedExtrinsic::<avail::data_availability::tx::SubmitData>::try_from(&data).unwrap();
	println!(
		"Signed Extrinsic DataAvailability::SubmitData data: {}, Tip: {}",
		String::from_utf8(extrinsic.call.data).unwrap(),
		extrinsic.signature.extra.tip
	);

	Ok(())
}

pub fn printout_details(infos: Vec<ExtrinsicInfo>) {
	for info in infos {
		println!(
			"Index: {}, Hash: {:?}, Pallet ID: {}, Variant ID: {}, App ID: {:?}, SS58 Address: {:?}, Data: {:?}",
			info.ext_index,
			info.ext_hash,
			info.pallet_id,
			info.variant_id,
			info.signer_payload.as_ref().map(|x| x.app_id),
			info.signer_payload.as_ref().map(|x| x.ss58_address.clone()).flatten(),
			info.data,
		);
	}
}

/*
	Expected Output:

	Number of extrinsics in block 2470159: 8
	Index: 0, Hash: 0x5627989c7e34303f78753e0bada2b9c626fc08a562fecdbe8562140272502818, Pallet ID: 3, Variant ID: 0, App ID: None, SS58 Address: None, Data: None
	Index: 1, Hash: 0xe1ed26bcdc700418e4629af82065a1d99fb6491c0ceaccade0300d1ee42e6a5c, Pallet ID: 29, Variant ID: 1, App ID: Some(1), SS58 Address: Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk"), Data: None
	Index: 2, Hash: 0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5, Pallet ID: 29, Variant ID: 1, App ID: Some(2), SS58 Address: Some("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ"), Data: None
	Index: 3, Hash: 0x6baab0e3ab7e11007dc952d1e2fdbc7031279315438df54e0be10304214c4ee4, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CPeyHASCF938Zi8NER26czZCpNfX6HPRpCiw5iZAXsY4wpq"), Data: None
	Index: 4, Hash: 0x58706d6b50934a572eeb6f261ee9e05d5e6c1d50869fc412aaf5336c47898c82, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA"), Data: None
	Index: 5, Hash: 0x6750897f5996257c4aad0edd972f2b27442b8ee52e06b10fae07778ac9f6cc46, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CyjVVqLznydpJ6zU2QKtpu1ZVcoxg2GWD4qc3npwdaPuVyq"), Data: None
	Index: 6, Hash: 0x6a06c3db2e4f6f933ef9d6ceeee237f75b77b031cdfc3eec33160727c06f2497, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5H5wVAbv1unXga1eKKdc9mC3UHjLuU8fLyj35jJPf9SFdVYm"), Data: None
	Index: 7, Hash: 0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218, Pallet ID: 39, Variant ID: 11, App ID: None, SS58 Address: None, Data: None

	Number of extrinsics in block 2470159 with Extrinsic Index 1: 1
	Index: 1, Hash: 0xe1ed26bcdc700418e4629af82065a1d99fb6491c0ceaccade0300d1ee42e6a5c, Pallet ID: 29, Variant ID: 1, App ID: Some(1), SS58 Address: Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk"), Data: None

	Number of extrinsics in block 2470159 with Extrinsic Hash 0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5: 1
	Index: 2, Hash: 0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5, Pallet ID: 29, Variant ID: 1, App ID: Some(2), SS58 Address: Some("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ"), Data: None

	Number of extrinsics in block 2470159 with tx index filter: 2
	Index: 1, Hash: 0xe1ed26bcdc700418e4629af82065a1d99fb6491c0ceaccade0300d1ee42e6a5c, Pallet ID: 29, Variant ID: 1, App ID: Some(1), SS58 Address: Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk"), Data: None
	Index: 3, Hash: 0x6baab0e3ab7e11007dc952d1e2fdbc7031279315438df54e0be10304214c4ee4, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CPeyHASCF938Zi8NER26czZCpNfX6HPRpCiw5iZAXsY4wpq"), Data: None

	Number of extrinsics in block 2470159 with tx hash filter: 2
	Index: 2, Hash: 0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5, Pallet ID: 29, Variant ID: 1, App ID: Some(2), SS58 Address: Some("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ"), Data: None
	Index: 7, Hash: 0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218, Pallet ID: 39, Variant ID: 11, App ID: None, SS58 Address: None, Data: None

	Number of extrinsics in block 2470159 with pallet id 29: 6
	Index: 1, Hash: 0xe1ed26bcdc700418e4629af82065a1d99fb6491c0ceaccade0300d1ee42e6a5c, Pallet ID: 29, Variant ID: 1, App ID: Some(1), SS58 Address: Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk"), Data: None
	Index: 2, Hash: 0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5, Pallet ID: 29, Variant ID: 1, App ID: Some(2), SS58 Address: Some("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ"), Data: None
	Index: 3, Hash: 0x6baab0e3ab7e11007dc952d1e2fdbc7031279315438df54e0be10304214c4ee4, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CPeyHASCF938Zi8NER26czZCpNfX6HPRpCiw5iZAXsY4wpq"), Data: None
	Index: 4, Hash: 0x58706d6b50934a572eeb6f261ee9e05d5e6c1d50869fc412aaf5336c47898c82, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA"), Data: None
	Index: 5, Hash: 0x6750897f5996257c4aad0edd972f2b27442b8ee52e06b10fae07778ac9f6cc46, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CyjVVqLznydpJ6zU2QKtpu1ZVcoxg2GWD4qc3npwdaPuVyq"), Data: None
	Index: 6, Hash: 0x6a06c3db2e4f6f933ef9d6ceeee237f75b77b031cdfc3eec33160727c06f2497, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5H5wVAbv1unXga1eKKdc9mC3UHjLuU8fLyj35jJPf9SFdVYm"), Data: None

	Number of extrinsics in block 2470159 with pallet id 29 and variant id 100: 0

	Number of extrinsics in block 2470159 with pallet id 3 and 39: 2
	Index: 0, Hash: 0x5627989c7e34303f78753e0bada2b9c626fc08a562fecdbe8562140272502818, Pallet ID: 3, Variant ID: 0, App ID: None, SS58 Address: None, Data: None
	Index: 7, Hash: 0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218, Pallet ID: 39, Variant ID: 11, App ID: None, SS58 Address: None, Data: None

	Number of extrinsics in block 2470159 with PV 29/100 or 3/0: 1
	Index: 0, Hash: 0x5627989c7e34303f78753e0bada2b9c626fc08a562fecdbe8562140272502818, Pallet ID: 3, Variant ID: 0, App ID: None, SS58 Address: None, Data: None

	Number of extrinsics in block 2470159 with app id 246: 4
	Index: 3, Hash: 0x6baab0e3ab7e11007dc952d1e2fdbc7031279315438df54e0be10304214c4ee4, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CPeyHASCF938Zi8NER26czZCpNfX6HPRpCiw5iZAXsY4wpq"), Data: None
	Index: 4, Hash: 0x58706d6b50934a572eeb6f261ee9e05d5e6c1d50869fc412aaf5336c47898c82, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA"), Data: None
	Index: 5, Hash: 0x6750897f5996257c4aad0edd972f2b27442b8ee52e06b10fae07778ac9f6cc46, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CyjVVqLznydpJ6zU2QKtpu1ZVcoxg2GWD4qc3npwdaPuVyq"), Data: None
	Index: 6, Hash: 0x6a06c3db2e4f6f933ef9d6ceeee237f75b77b031cdfc3eec33160727c06f2497, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5H5wVAbv1unXga1eKKdc9mC3UHjLuU8fLyj35jJPf9SFdVYm"), Data: None

	Number of extrinsics in block 2470159 with nonce 2221: 1
	Index: 3, Hash: 0x6baab0e3ab7e11007dc952d1e2fdbc7031279315438df54e0be10304214c4ee4, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5CPeyHASCF938Zi8NER26czZCpNfX6HPRpCiw5iZAXsY4wpq"), Data: None

	Number of extrinsics in block 2470159 with ss58 address 5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA: 1
	Index: 4, Hash: 0x58706d6b50934a572eeb6f261ee9e05d5e6c1d50869fc412aaf5336c47898c82, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA"), Data: None

	Number of extrinsics in block 2470159 with app id 246 and address 5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA: 1
	Index: 4, Hash: 0x58706d6b50934a572eeb6f261ee9e05d5e6c1d50869fc412aaf5336c47898c82, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA"), Data: None

	Number of data submission extrinsics in block 2470159 with app id 246 and address 5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA: 1
	Index: 4, Hash: 0x58706d6b50934a572eeb6f261ee9e05d5e6c1d50869fc412aaf5336c47898c82, Pallet ID: 29, Variant ID: 1, App ID: Some(246), SS58 Address: Some("5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA"), Data: None

	Index: 0, Hash: 0x5627989c7e34303f78753e0bada2b9c626fc08a562fecdbe8562140272502818, Pallet ID: 3, Variant ID: 0, App ID: None, SS58 Address: None, Data: Some("03000b0092660c9a01")
	Index: 1, Hash: 0xe1ed26bcdc700418e4629af82065a1d99fb6491c0ceaccade0300d1ee42e6a5c, Pallet ID: 29, Variant ID: 1, App ID: Some(1), SS58 Address: Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk"), Data: Some("1d01144558542031")
	Timestamp::Set now: 1761144640000
	DataAvailability::SubmitData data: EXT 1

	Index: 0, Hash: 0x5627989c7e34303f78753e0bada2b9c626fc08a562fecdbe8562140272502818, Pallet ID: 3, Variant ID: 0, App ID: None, SS58 Address: None, Data: Some("280403000b0092660c9a01")
	Index: 1, Hash: 0xe1ed26bcdc700418e4629af82065a1d99fb6491c0ceaccade0300d1ee42e6a5c, Pallet ID: 29, Variant ID: 1, App ID: Some(1), SS58 Address: Some("5DPDXCcqk1YNVZ3M9s9iwJnr9XAVfTxf8hNa4LS51fjHKAzk"), Data: Some("c10184003a5a8284d650213a9e29f4b87efdb1f6c119fbd0e4f4838c39ec2beb6a3409390130e45f934e82212e0a6fb9cc198ee21622cbd651367afc97ce144c34bce21125d79242c57ee4f021bbd350afa5c0483a0bc53a3e6672454116747b0c76deb385d4000800041d01144558542031")
	Encoded Extrinsic Timestamp::Set call length: 9, Tip: None
	Extrinsic Timestamp::Set now: 1761144640000, Tip: None
	Encoded Extrinsic DataAvailability::SubmitData call length: 8, Tip: Some(0)
	Extrinsic DataAvailability::SubmitData data: EXT 1, Tip: Some(0)
	Signed Extrinsic DataAvailability::SubmitData data: EXT 1, Tip: 0
*/
