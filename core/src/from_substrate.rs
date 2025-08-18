use codec::{Decode, Encode};
use serde::Deserialize;

/// The base fee and adjusted weight and length fees constitute the _inclusion fee_.
#[derive(Clone, Debug, PartialEq, Deserialize, Decode)]
#[serde(rename_all = "camelCase")]
pub struct InclusionFee {
	/// This is the minimum amount a user pays for a transaction. It is declared
	/// as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
	pub base_fee: u128,
	/// The length fee, the amount paid for the encoded length (in bytes) of the transaction.
	pub len_fee: u128,
	///
	/// - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on the
	///   congestion of the network.
	/// - `weight_fee`: This amount is computed based on the weight of the transaction. Weight
	///	  accounts for the execution time of a transaction.
	///
	/// adjusted_weight_fee = targeted_fee_adjustment * weight_fee
	pub adjusted_weight_fee: u128,
}

impl InclusionFee {
	/// Returns the total of inclusion fee.
	///
	/// ```ignore
	/// inclusion_fee = base_fee + len_fee + adjusted_weight_fee
	/// ```
	pub fn inclusion_fee(&self) -> u128 {
		self.base_fee
			.saturating_add(self.len_fee)
			.saturating_add(self.adjusted_weight_fee)
	}
}

/// The `FeeDetails` is composed of:
///   - (Optional) `inclusion_fee`: Only the `Pays::Yes` transaction can have the inclusion fee.
///   - `tip`: If included in the transaction, the tip will be added on top. Only signed
///     transactions can have a tip.
#[derive(Clone, Debug, PartialEq, Deserialize, Decode)]
#[serde(rename_all = "camelCase")]
pub struct FeeDetails {
	/// The minimum fee for a transaction to be included in a block.
	pub inclusion_fee: Option<InclusionFee>,
	// Do not serialize and deserialize `tip` as we actually can not pass any tip to the RPC.
	#[codec(skip)]
	pub tip: u128,
}

impl FeeDetails {
	/// Returns the final fee.
	///
	/// ```ignore
	/// final_fee = inclusion_fee + tip;
	/// ```
	pub fn final_fee(&self) -> u128 {
		self.inclusion_fee
			.as_ref()
			.map(|i| i.inclusion_fee())
			.unwrap_or(0)
			.saturating_add(self.tip)
	}
}

#[derive(Clone, Debug, PartialEq, Deserialize, Decode)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDispatchInfo {
	/// Weight of this dispatch.
	pub weight: Weight,
	/// Class of this dispatch.
	pub class: DispatchClass,
	pub partial_fee: u128,
}

#[derive(Clone, Debug, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum DispatchClass {
	/// A normal dispatch.
	Normal = 0,
	/// An operational dispatch.
	Operational = 1,
	/// A mandatory dispatch. These kinds of dispatch are always included regardless of their
	/// weight, therefore it is critical that they are separately validated to ensure that a
	/// malicious validator cannot craft a valid but impossibly heavy block. Usually this just
	/// means ensuring that the extrinsic can only be included once and that it is always very
	/// light.
	///
	/// Do *NOT* use it for extrinsics that can be heavy.
	///
	/// The only real use case for this is inherent extrinsics that are required to execute in a
	/// block for the block to be valid, and it solves the issue in the case that the block
	/// initialization is sufficiently heavy to mean that those inherents do not fit into the
	/// block. Essentially, we assume that in these exceptional circumstances, it is better to
	/// allow an overweight block to be created than to not allow any block at all to be created.
	Mandatory = 2,
}
impl Encode for DispatchClass {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		let variant: u8 = *self as u8;
		variant.encode_to(dest);
	}
}
impl Decode for DispatchClass {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let variant = u8::decode(input)?;
		match variant {
			0 => Ok(Self::Normal),
			1 => Ok(Self::Operational),
			2 => Ok(Self::Mandatory),
			_ => Err("Failed to decode DispatchClass. Unknown variant".into()),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Encode, Decode)]
pub struct Weight {
	/// The weight of computational time used based on some reference hardware.
	#[codec(compact)]
	pub ref_time: u64,
	/// The weight of storage space used by proof of validity.
	#[codec(compact)]
	pub proof_size: u64,
}
