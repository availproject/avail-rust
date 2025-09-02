//! This example showcases the following actions:
//! - Transaction Creation
//! - Transaction Submission
//! - Fetching Transaction Receipt
//! - Fetching Block State
//! - Fetching and displaying Transaction Events
//! - Fetching Block Transaction
//!

use avail_rust_client::prelude::*;
use codec::{Compact, Decode, Encode};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let a = AvailHeader {
		parent_hash: Default::default(),
		number: Default::default(),
		state_root: Default::default(),
		extrinsics_root: Default::default(),
		digest: Default::default(),
		extension: HeaderExtension::V3(V3HeaderExtension {
			app_lookup: Default::default(),
			commitment: KateCommitment {
				rows: 4,
				cols: 4,
				commitment: vec![1u8; 1000 * 1000 * 250],
				data_root: H256::from([5u8; 32]),
			},
		}),
	};

	let mut buffer: Vec<u8> = Vec::new();
	a.encode_to(&mut buffer);
	/* 	a.encode_to(&mut buffer);
	   dbg!(&buffer);
	   dbg!(buffer.len());

	   for _ in 0..32 {
		   buffer.pop();
	   }
	   buffer.pop();
	   buffer.pop();
	   dbg!(&buffer);
	   Compact(1000u32 * 1000 * 100).encode_to(&mut buffer);
	   [1u8; 10_000].encode_to(&mut buffer);
	*/
	let mut b = buffer.as_slice();
	dbg!(b.len());
	let b = AvailHeader::decode(&mut b)?;

	/* 	Compact::<u32>(1000 * 1000 * 100).encode_to(&mut buffer);
	buffer.extend(HeavyStruct { number: 5, image: [1u32; 100], house: [2u8; 8000] }.encode()); */
	// buffer.extend(HeavyStruct { number: 5, image: [3u32; 100], house: [4u8; 8000] }.encode());

	/* 	let mut b = buffer.as_slice();

	dbg!(a); */

	Ok(())
}
