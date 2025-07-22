use crate::AccountId;
use codec::{Codec, Decode};
use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::AvailHeader;

#[derive(Debug, Clone, Decode)]
pub struct AuthorityId(pub [u8; 32]);
pub type Public = AuthorityId;

impl Serialize for AuthorityId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let account_id = AccountId::from(self.0.clone());
		serializer.serialize_str(&account_id.to_string())
	}
}

impl<'de> Deserialize<'de> for AuthorityId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let account_id = AccountId::deserialize(deserializer)?;
		Ok(Self(account_id.0))
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
impl Serialize for Signature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&const_hex::encode(&self.0))
	}
}

impl<'de> Deserialize<'de> for Signature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let signature_hex = const_hex::decode(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		let signature: [u8; 64usize] = signature_hex
			.try_into()
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		Ok(Self(signature))
	}
}

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
