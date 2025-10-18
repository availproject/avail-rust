use avail_rust_client::{
	avail::{
		data_availability::{events::DataSubmitted, tx::SubmitData},
		timestamp::tx::Set,
	},
	prelude::*,
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	/*
		Part 1: Block Structure - The Theory

		Blocks are a fundamental piece of Avail Blockchain.
		Every block has the same structure:
			1. An array of executed extrinsics
			2. [Optional] GRANDPA justification proof

		Blocks and extrinsics inside a block emit events like System::ExtrinsicSuccess or DataAvailability::DataSubmitted,
		but the events themselves are not stored inside a block. A block will tell us that a change has occur, for example
		a balance transfer between Alice and Bob, but it will not tell us the final state of their accounts. To get that,
		same goes for events, you need to query the blockchain state storage at a specific block hash.

		The extrinsics inside a block are SCALE and HEX encoded when fetched from a node. This means that in order to find
		out what change has occurred we need to decode all extrinsics no matter if we are interested in only a single one or
		ones that have `app_id` set to 3. If we know upfront that our target extrinsics are at specific positions, then we
		just need to decode the ones at these positions. A position inside an extrinsic array is called `extrinsic index`.

		Part 2: Extrinsic Structure

		An extrinsic is made out of two parts:
			1. [Optional] Extrinsic Signature
				I. Address (Enum)
				II. Signature (Enum)
				III. Extra
					1. Era
					2. Nonce (Compact u32)
					3. Tip (Compact u128)
					4. App ID (Compact u32)
			2. Extrinsic Call
				I. Header
					1. Pallet ID (u8)
					2. Variant ID (u8)
				II. Data (Array of SCALE encoded bytes)

		Unsigned extrinsics are extrinsics that do not have a `Extrinsic Signature` and these extrinsics are usually emitted by
		the chain itself. One example is the Timestamp::Set extrinsic that embeds a timestamp to each block. This extrinsic is
		emitted on every block and it is always the first extrinsic

		Signed extrinsics are extrinsics that do have a `Extrinsic Signature` and these extrinsics are always submitted by an
		account. One example is the Balances::TransferKeepAlive extrinsic that allows us send tokens between accounts. Signed
		extrinsics are as well know as transactions. As most of interesting (to us) extrinsics are signed, it is common to
		use the word transaction instead of the word extrinsic for all types of extrinsics even thought that might not be technically correct.

		Encoded extrinsics are extrinsics that are opaque to us and we don't know the content of it. These extrinsics come in two
		flavours:
			1. Fully Encoded
				- Here we deal with an array of bytes
			2. Partially Encoded
				- Here we deal with the `Extrinsic Signature` decoded but the `Extrinsic Call` is still SCALE encoded


		Part 3: Event Structure

		An event is made out of three parts:
			1. Phase (Enum)
			2. Event Data
				I. Header
					1. Pallet ID (u8)
					2. Variant ID (u8)
				II. Call (Array of SCALE encoded bytes)
			3. Topics

		Event Phase has the following variants:
			- ApplyExtrinsic(u32)
			- Finalization
			- Initialization

		Not all events inside a block are emitted by extrinsics. Some events are block related and they use `Finalization` or
		`Initialization` as their `Phase` variant.

		Events are stored inside the System::Events storage as a array of SCALE encoded bytes. Fetching and then decoding the events
		from the storage is a problematic task. With extrinsics inside a block, you deal with an array of encoded bytes and then
		you can decode just the ones at specific positions. With events this is not possible so in order to find an event you
		need to decode all events altogether. This means that if your extrinsic was executed as the last one then events
		related to this extrinsic will be at the end of the encoded bytes, but to decode the end you need to decode everything
		that came before. We cannot skip or guess properly as events have dynamic sizes and the size is not known to us.
	*/

	Ok(())
}
