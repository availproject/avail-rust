use crate::substrate::sp_core::vrf::VrfSignature;
use codec::{Decode, Encode};
pub use subxt_core::config::substrate::ConsensusEngineId;

pub mod babe {
	use super::*;

	/// The `ConsensusEngineId` of BABE.
	pub const BABE_ENGINE_ID: ConsensusEngineId = *b"BABE";

	/// Raw BABE primary slot assignment pre-digest.
	#[derive(Clone, Encode, Decode, Debug)]
	pub struct PrimaryPreDigest {
		/// Authority index
		pub authority_index: u32,
		/// Slot
		pub slot: u64,
		/// VRF signature
		pub vrf_signature: VrfSignature,
	}

	/// BABE secondary slot assignment pre-digest.
	#[derive(Clone, Encode, Decode, Debug)]
	pub struct SecondaryPlainPreDigest {
		/// Authority index
		///
		/// This is not strictly-speaking necessary, since the secondary slots
		/// are assigned based on slot number and epoch randomness. But including
		/// it makes things easier for higher-level users of the chain data to
		/// be aware of the author of a secondary-slot block.
		pub authority_index: u32,
		/// Slot
		pub slot: u64,
	}

	/// BABE secondary deterministic slot assignment with VRF outputs.
	#[derive(Clone, Encode, Decode, Debug)]
	pub struct SecondaryVRFPreDigest {
		/// Authority index
		pub authority_index: u32,
		/// Slot
		pub slot: u64,
		/// VRF signature
		pub vrf_signature: VrfSignature,
	}

	/// A BABE pre-runtime digest. This contains all data required to validate a
	/// block and for the BABE runtime module. Slots can be assigned to a primary
	/// (VRF based) and to a secondary (slot number based).
	#[derive(Clone, Encode, Decode, Debug)]
	pub enum PreDigest {
		/// A primary VRF-based slot assignment.
		#[codec(index = 1)]
		Primary(PrimaryPreDigest),
		/// A secondary deterministic slot assignment.
		#[codec(index = 2)]
		SecondaryPlain(SecondaryPlainPreDigest),
		/// A secondary deterministic slot assignment with VRF outputs.
		#[codec(index = 3)]
		SecondaryVRF(SecondaryVRFPreDigest),
	}

	impl PreDigest {
		/// Returns the slot number of the pre digest.
		pub fn authority_index(&self) -> u32 {
			match self {
				PreDigest::Primary(primary) => primary.authority_index,
				PreDigest::SecondaryPlain(secondary) => secondary.authority_index,
				PreDigest::SecondaryVRF(secondary) => secondary.authority_index,
			}
		}

		/// Returns the slot of the pre digest.
		pub fn slot(&self) -> u64 {
			match self {
				PreDigest::Primary(primary) => primary.slot,
				PreDigest::SecondaryPlain(secondary) => secondary.slot,
				PreDigest::SecondaryVRF(secondary) => secondary.slot,
			}
		}

		/// Returns true if this pre-digest is for a primary slot assignment.
		pub fn is_primary(&self) -> bool {
			matches!(self, PreDigest::Primary(..))
		}

		/// Returns the VRF output and proof, if they exist.
		pub fn vrf_signature(&self) -> Option<&VrfSignature> {
			match self {
				PreDigest::Primary(primary) => Some(&primary.vrf_signature),
				PreDigest::SecondaryVRF(secondary) => Some(&secondary.vrf_signature),
				PreDigest::SecondaryPlain(_) => None,
			}
		}
	}
}
