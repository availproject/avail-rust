use avail_rust_client::prelude::*;
use avail_rust_core::{
	decoded_transaction::SignedExtrinsic,
	rpc::{ExtrinsicInfo, system::fetch_extrinsics::Options},
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);
	let opts = Options::new().encode_as(EncodeSelector::None);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159: {}", infos.len());
	printout_info_details(infos);
	println!("");

	// 1
	let opts = Options::new().encode_as(EncodeSelector::None).filter(1u32);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with Extrinsic Index 1: {}", infos.len());
	printout_info_details(infos);
	println!("");

	let ext_hash = H256::from_str("0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5").unwrap();
	let opts = Options::new().encode_as(EncodeSelector::None).filter(ext_hash);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with Extrinsic Hash {:?}: {}", ext_hash, infos.len());
	printout_info_details(infos);
	println!("");

	// 2
	let opts = Options::new().encode_as(EncodeSelector::None).filter(vec![1u32, 3u32]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with filter: {}", infos.len());
	printout_info_details(infos);
	println!("");

	let ext_hash_1 = H256::from_str("0xede18e2b5714cf4f77b94fab2e1ab45b815da1af88914cc950c0d4eff7c5eef5").unwrap();
	let ext_hash_2 = H256::from_str("0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218").unwrap();
	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.filter(vec![ext_hash_1, ext_hash_2]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with filter: {}", infos.len());
	printout_info_details(infos);
	println!("");

	// 3
	let opts = Options::new().encode_as(EncodeSelector::None).filter(29u8);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with pallet id 29: {}", infos.len());
	printout_info_details(infos);
	println!("");

	let opts = Options::new().encode_as(EncodeSelector::None).filter((29u8, 100u8));
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with pallet id 29 and variant id 100: {}", infos.len());
	printout_info_details(infos);
	println!("");

	// 4
	let opts = Options::new().encode_as(EncodeSelector::None).filter(vec![3u8, 39u8]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with pallet id 3 and 39: {}", infos.len());
	printout_info_details(infos);
	println!("");

	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.filter(vec![(29u8, 100u8), (3u8, 0u8)]);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with filter: {}", infos.len());
	printout_info_details(infos);
	println!("");

	// 5
	let opts = Options::new().encode_as(EncodeSelector::None).app_id(246);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with app id 246: {}", infos.len());
	printout_info_details(infos);
	println!("");

	let opts = Options::new().encode_as(EncodeSelector::None).nonce(2221);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with nonce 2160: {}", infos.len());
	printout_info_details(infos);
	println!("");

	let address = "5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA";
	let opts = Options::new().encode_as(EncodeSelector::None).ss58_address(address);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with ss58 address {}: {}", address, infos.len());
	printout_info_details(infos);
	println!("");

	// 6
	let address = "5E9MGdHYokTQzhhfPhfFXfvyMVVnmjdYLy6DcG78srBnYZLA";
	let opts = Options::new()
		.encode_as(EncodeSelector::None)
		.app_id(246)
		.ss58_address(address);
	let infos = block.extrinsic_infos(opts).await?;
	println!("Number of extrinsics in block 2470159 with app id 246 and address {}: {}", address, infos.len());
	printout_info_details(infos);
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
	printout_info_details(infos);
	println!("");

	// 8
	let opts = Options::new().encode_as(EncodeSelector::Call).filter(vec![0u32, 1u32]);
	let mut infos = block.extrinsic_infos(opts).await?;
	printout_info_details(infos.clone());

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
	printout_info_details(infos.clone());

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

pub fn printout_info_details(infos: Vec<ExtrinsicInfo>) {
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
