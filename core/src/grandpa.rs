use codec::{Codec, Decode};
use serde::{Serialize, Serializer};

#[cfg(feature = "generated_metadata")]
use crate::avail_generated::runtime_types::sp_consensus_grandpa::app::Public;

#[cfg(not(feature = "generated_metadata"))]
pub type Public = [u8; 32];

#[derive(Decode)]
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
