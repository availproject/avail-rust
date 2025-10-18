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

		Blocks are a fundamental piece of the Avail blockchain.
		Every block shares the same structure:
			1. An array of executed extrinsics
			2. [Optional] GRANDPA justification proof

		Blocks and the extrinsics inside them emit events like System::ExtrinsicSuccess or DataAvailability::DataSubmitted,
		but the events themselves are not stored in the block. A block will tell us that a change has occurred, for example
		a balance transfer between Alice and Bob, but it will not tell us the final state of their accounts. To get that information
		(the same goes for events), you need to query the blockchain state storage at a specific block hash.

		The extrinsics inside a block are SCALE- and hex-encoded when fetched from a node. This means that, in order to find
		out what change has occurred, we need to decode all extrinsics no matter whether we are interested in only a single one or
		in those that have `app_id` set to 3. If we know up front that our target extrinsics are at specific positions, then we
		just need to decode the ones at these positions. A position inside an extrinsic array is called an `extrinsic index`.

		Part 2: Extrinsic Structure

		An extrinsic consists of two parts:
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
				II. Data (Array of SCALE-encoded bytes)

		Unsigned extrinsics are extrinsics that do not have an `Extrinsic Signature`, and these extrinsics are usually emitted by
		the chain itself. One example is the Timestamp::Set extrinsic that embeds a timestamp in each block. This extrinsic is
		emitted on every block, and it is always the first extrinsic.

		Signed extrinsics are extrinsics that do have an `Extrinsic Signature`, and these extrinsics are always submitted by an
		account. One example is the Balances::TransferKeepAlive extrinsic that allows us to send tokens between accounts. Signed
		extrinsics are also known as transactions. As most of the interesting (to us) extrinsics are signed, it is common to
		use the word transaction instead of the word extrinsic for all types of extrinsics even though that might not be technically correct.

		Encoded extrinsics are extrinsics that are opaque to us, and we don't know their content. These extrinsics come in two
		flavours:
			1. Fully Encoded
				- Here we deal with an array of bytes
			2. Partially Encoded
				- Here we deal with the `Extrinsic Signature` decoded, but the `Extrinsic Call` is still SCALE-encoded


		Part 3: Event Structure

		An event consists of three parts:
			1. Phase (Enum)
			2. Event Data
				I. Header
					1. Pallet ID (u8)
					2. Variant ID (u8)
				II. Call (Array of SCALE-encoded bytes)
			3. Topics

		Event Phase has the following variants:
			- ApplyExtrinsic(u32)
			- Finalization
			- Initialization

		Not all events inside a block are emitted by extrinsics. Some events are block related, and they use `Finalization` or
		`Initialization` as their `Phase` variant.

		Events live in `System::Events` as a single SCALE-encoded blob. Fetching this storage key gives you one long byte array.
		Unlike extrinsics, you cannot jump straight to a specific entry because every event has a variable size. You must decode
		the stream sequentially from the beginning until you reach the event you care about. If the extrinsic you are tracking
		was executed last, its events will appear near the end of the blob, but you still need to process everything that comes
		before it.
	*/

	Ok(())
}
