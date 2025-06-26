use codec::{Codec, Decode};
use primitive_types::H256;
use serde::{Serialize, Serializer};

#[cfg(feature = "generated_metadata")]
use crate::avail_generated::runtime_types::sp_consensus_grandpa::app::Public;
use crate::AvailHeader;

#[cfg(not(feature = "generated_metadata"))]
pub type Public = [u8; 32];

#[derive(Debug, Clone, Decode)]
pub struct AuthorityId(pub Public);

impl Serialize for AuthorityId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		#[cfg(feature = "generated_metadata")]
		{
			let raw = hex::encode(self.0 .0 .0);
			serializer.serialize_str(&raw)
		}

		#[cfg(not(feature = "generated_metadata"))]
		{
			let raw = hex::encode(self.0);
			serializer.serialize_str(&raw)
		}
	}
}

pub type AuthorityIndex = u64;
pub type AuthorityWeight = u64;
pub type AuthorityList = Vec<(AuthorityId, AuthorityWeight)>;

#[derive(Decode, Serialize)]
pub struct ScheduledChange<N> {
	/// The new authorities after the change, along with their respective weights.
	pub next_authorities: AuthorityList,
	/// The number of blocks to delay.
	pub delay: N,
}
/// An consensus log item for GRANDPA.
#[derive(Decode, Serialize)]
#[repr(u8)]
pub enum ConsensusLog<N: Codec> {
	ScheduledChange(ScheduledChange<N>) = 1,
	ForcedChange(N, ScheduledChange<N>) = 2,
	OnDisabled(AuthorityIndex) = 3,
	Pause(N) = 4,
	Resume(N) = 5,
}

#[derive(Debug, Clone, Copy, Decode)]
pub struct Signature(pub [u8; 64usize]);

#[derive(Debug, Clone, codec::Decode)]
pub struct Precommit {
	/// The target block's hash.
	pub target_hash: H256,
	/// The target block's number
	pub target_number: u32,
}

#[derive(Debug, Clone, codec::Decode)]
pub struct SignedPrecommit {
	/// The precommit message which has been signed.
	pub precommit: Precommit,
	/// The signature on the message.
	pub signature: Signature,
	/// The Id of the signer.
	pub id: AuthorityId,
}

#[derive(Debug, Clone, codec::Decode)]
pub struct Commit {
	/// The target block's hash.
	pub target_hash: H256,
	/// The target block's number.
	pub target_number: u32,
	/// Precommits for target block or any block after it that justify this commit.
	pub precommits: Vec<SignedPrecommit>,
}

#[derive(Debug, Clone, codec::Decode)]
pub struct GrandpaJustification {
	pub round: u64,
	pub commit: Commit,
	pub votes_ancestries: Vec<AvailHeader>,
}
