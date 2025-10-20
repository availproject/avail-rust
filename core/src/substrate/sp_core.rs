// # This code is adapted from the Substrate project
// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0
// See http://www.apache.org/licenses/LICENSE-2.0 for details.

use codec::{Decode, Encode};

/// Schnorrkel VRF related types and operations.
pub mod vrf {
	use super::*;
	use schnorrkel::{
		SignatureError,
		errors::MultiSignatureStage,
		vrf::{VRF_PREOUT_LENGTH, VRF_PROOF_LENGTH},
	};

	/// VRF signature data
	#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
	pub struct VrfSignature {
		/// VRF pre-output.
		pub pre_output: VrfPreOutput,
		/// VRF proof.
		pub proof: VrfProof,
	}

	/// VRF pre-output type suitable for schnorrkel operations.
	#[derive(Clone, Debug, PartialEq, Eq)]
	pub struct VrfPreOutput(pub schnorrkel::vrf::VRFPreOut);

	impl Encode for VrfPreOutput {
		fn encode(&self) -> Vec<u8> {
			self.0.as_bytes().encode()
		}
	}

	impl Decode for VrfPreOutput {
		fn decode<R: codec::Input>(i: &mut R) -> Result<Self, codec::Error> {
			let decoded = <[u8; VRF_PREOUT_LENGTH]>::decode(i)?;
			Ok(Self(schnorrkel::vrf::VRFPreOut::from_bytes(&decoded).map_err(convert_error)?))
		}
	}

	/// VRF proof type suitable for schnorrkel operations.
	#[derive(Clone, Debug, PartialEq, Eq)]
	pub struct VrfProof(pub schnorrkel::vrf::VRFProof);

	impl Encode for VrfProof {
		fn encode(&self) -> Vec<u8> {
			self.0.to_bytes().encode()
		}
	}

	impl Decode for VrfProof {
		fn decode<R: codec::Input>(i: &mut R) -> Result<Self, codec::Error> {
			let decoded = <[u8; VRF_PROOF_LENGTH]>::decode(i)?;
			Ok(Self(schnorrkel::vrf::VRFProof::from_bytes(&decoded).map_err(convert_error)?))
		}
	}

	fn convert_error(e: SignatureError) -> codec::Error {
		use MultiSignatureStage::*;
		use SignatureError::*;
		match e {
			EquationFalse => "Signature error: `EquationFalse`".into(),
			PointDecompressionError => "Signature error: `PointDecompressionError`".into(),
			ScalarFormatError => "Signature error: `ScalarFormatError`".into(),
			NotMarkedSchnorrkel => "Signature error: `NotMarkedSchnorrkel`".into(),
			BytesLengthError { .. } => "Signature error: `BytesLengthError`".into(),
			InvalidKey => "Signature error: `InvalidKey`".into(),
			MuSigAbsent { musig_stage: Commitment } => "Signature error: `MuSigAbsent` at stage `Commitment`".into(),
			MuSigAbsent { musig_stage: Reveal } => "Signature error: `MuSigAbsent` at stage `Reveal`".into(),
			MuSigAbsent { musig_stage: Cosignature } => "Signature error: `MuSigAbsent` at stage `Commitment`".into(),
			MuSigInconsistent { musig_stage: Commitment, duplicate: true } => {
				"Signature error: `MuSigInconsistent` at stage `Commitment` on duplicate".into()
			},
			MuSigInconsistent { musig_stage: Commitment, duplicate: false } => {
				"Signature error: `MuSigInconsistent` at stage `Commitment` on not duplicate".into()
			},
			MuSigInconsistent { musig_stage: Reveal, duplicate: true } => {
				"Signature error: `MuSigInconsistent` at stage `Reveal` on duplicate".into()
			},
			MuSigInconsistent { musig_stage: Reveal, duplicate: false } => {
				"Signature error: `MuSigInconsistent` at stage `Reveal` on not duplicate".into()
			},
			MuSigInconsistent { musig_stage: Cosignature, duplicate: true } => {
				"Signature error: `MuSigInconsistent` at stage `Cosignature` on duplicate".into()
			},
			MuSigInconsistent { musig_stage: Cosignature, duplicate: false } => {
				"Signature error: `MuSigInconsistent` at stage `Cosignature` on not duplicate".into()
			},
		}
	}
}
