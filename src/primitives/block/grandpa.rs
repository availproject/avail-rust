use codec::{Codec, Decode};
use serde::{Serialize, Serializer};

#[cfg(feature = "subxt")]
use crate::avail::runtime_types::sp_consensus_grandpa::app::Public;

#[cfg(not(feature = "subxt"))]
pub type Public = [u8; 32];

#[derive(Decode)]
pub struct AuthorityId(pub Public);

impl Serialize for AuthorityId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		todo!();
		//let raw = hex::encode(self.0 .0 .0);
		//serializer.serialize_str(&raw)
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
pub enum ConsensusLog<N: Codec> {
	#[codec(index = 1)]
	ScheduledChange(ScheduledChange<N>),
	#[codec(index = 2)]
	ForcedChange(N, ScheduledChange<N>),
	#[codec(index = 3)]
	OnDisabled(AuthorityIndex),
	#[codec(index = 4)]
	Pause(N),
	#[codec(index = 5)]
	Resume(N),
}
