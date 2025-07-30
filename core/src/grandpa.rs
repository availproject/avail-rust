use crate::{AccountId, AvailHeader};
use codec::{Codec, Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

pub type AuthorityIndex = u64;
pub type AuthorityWeight = u64;
pub type AuthorityList = Vec<(AuthorityId, AuthorityWeight)>;

#[derive(Debug, Clone)]
pub struct AuthorityId(pub [u8; 32]);
pub type Public = AuthorityId;

impl Encode for AuthorityId {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.0.encode_to(dest);
	}
}
impl Decode for AuthorityId {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		Ok(Self(Decode::decode(input)?))
	}
}
impl Serialize for AuthorityId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let account_id = AccountId::from(self.0);
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

#[derive(Debug, Clone, Serialize, Encode, Decode)]
pub struct ScheduledChange<N> {
	/// The new authorities after the change, along with their respective weights.
	pub next_authorities: AuthorityList,
	/// The number of blocks to delay.
	pub delay: N,
}
/// An consensus log item for GRANDPA.
#[derive(Debug, Clone, Serialize, Encode, Decode)]
#[repr(u8)]
pub enum ConsensusLog<N: Codec> {
	ScheduledChange(ScheduledChange<N>) = 1,
	ForcedChange(N, ScheduledChange<N>) = 2,
	OnDisabled(AuthorityIndex) = 3,
	Pause(N) = 4,
	Resume(N) = 5,
}

#[derive(Debug, Clone, Copy)]
pub struct Signature(pub [u8; 64usize]);
impl Encode for Signature {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.0.encode_to(dest);
	}
}
impl Decode for Signature {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		Ok(Self(Decode::decode(input)?))
	}
}
impl Serialize for Signature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&const_hex::encode(self.0))
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

#[derive(Debug, Clone)]
pub struct Precommit {
	/// The target block's hash.
	pub target_hash: H256,
	/// The target block's number
	pub target_number: u32,
}
impl Encode for Precommit {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.target_hash.encode_to(dest);
		self.target_number.encode_to(dest);
	}
}
impl Decode for Precommit {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let target_hash = Decode::decode(input)?;
		let target_number = Decode::decode(input)?;
		Ok(Self { target_hash, target_number })
	}
}

#[derive(Debug, Clone)]
pub struct SignedPrecommit {
	/// The precommit message which has been signed.
	pub precommit: Precommit,
	/// The signature on the message.
	pub signature: Signature,
	/// The Id of the signer.
	pub id: AuthorityId,
}
impl Encode for SignedPrecommit {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.precommit.encode_to(dest);
		self.signature.encode_to(dest);
		self.id.encode_to(dest);
	}
}
impl Decode for SignedPrecommit {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let precommit = Decode::decode(input)?;
		let signature = Decode::decode(input)?;
		let id = Decode::decode(input)?;
		Ok(Self { precommit, signature, id })
	}
}

#[derive(Debug, Clone)]
pub struct Commit {
	/// The target block's hash.
	pub target_hash: H256,
	/// The target block's number.
	pub target_number: u32,
	/// Precommits for target block or any block after it that justify this commit.
	pub precommits: Vec<SignedPrecommit>,
}
impl Encode for Commit {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.target_hash.encode_to(dest);
		self.target_number.encode_to(dest);
		self.precommits.encode_to(dest);
	}
}
impl Decode for Commit {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let target_hash = Decode::decode(input)?;
		let target_number = Decode::decode(input)?;
		let precommits = Decode::decode(input)?;
		Ok(Self { target_hash, target_number, precommits })
	}
}

#[derive(Debug, Clone)]
pub struct GrandpaJustification {
	pub round: u64,
	pub commit: Commit,
	pub votes_ancestries: Vec<AvailHeader>,
}
impl Encode for GrandpaJustification {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.round.encode_to(dest);
		self.commit.encode_to(dest);
		self.votes_ancestries.encode_to(dest);
	}
}
impl Decode for GrandpaJustification {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let round = Decode::decode(input)?;
		let commit = Decode::decode(input)?;
		let votes_ancestries = Decode::decode(input)?;
		Ok(Self { round, commit, votes_ancestries })
	}
}
