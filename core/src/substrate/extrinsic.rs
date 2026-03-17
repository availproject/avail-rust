use crate::{
	HasHeader,
	types::{AccountId, H256, MultiAddress, MultiSignature},
	utils::decode_already_decoded,
};
use codec::{Compact, CountedInput, Decode, Encode, Input};
use serde::{Deserialize, Serialize};
use subxt_core::{
	config::{Hasher, substrate::BlakeTwo256},
	utils::Era,
};
use subxt_signer::sr25519::Keypair;

pub type ExtensionVersion = u8;
pub type ExtrinsicVersion = u8;

pub const VERSION_MASK: u8 = 0b0011_1111;
pub const TYPE_MASK: u8 = 0b1100_0000;
pub const BARE_EXTRINSIC: u8 = 0b0000_0000;
pub const SIGNED_EXTRINSIC: u8 = 0b1000_0000;
pub const GENERAL_EXTRINSIC: u8 = 0b0100_0000;

pub const EXTENSION_VERSION: ExtensionVersion = 0;
pub const LEGACY_EXTRINSIC_FORMAT_VERSION: ExtrinsicVersion = 4;
pub const EXTRINSIC_FORMAT_VERSION: ExtrinsicVersion = 5;

#[derive(Clone, PartialEq, Eq)]
pub enum Preamble {
	/// An extrinsic without a signature or any extension. This means it's either an inherent or
	/// an old-school "Unsigned" (we don't use that terminology any more since it's confusable with
	/// the general transaction which is without a signature but does have an extension).
	///
	/// NOTE: In the future, once we remove `ValidateUnsigned`, this will only serve Inherent
	/// extrinsics and thus can be renamed to `Inherent`.
	Bare(ExtrinsicVersion),
	/// An old-school transaction extrinsic which includes a signature of some hard-coded crypto.
	/// Available only on extrinsic version 4.
	Signed(MultiAddress, MultiSignature, Extension),
	/// A new-school transaction extrinsic which does not include a signature by default. The
	/// origin authorization, through signatures or other means, is performed by the transaction
	/// extension in this extrinsic. Available starting with extrinsic version 5.
	General(ExtensionVersion, Extension),
}

impl Decode for Preamble {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let version_and_type = input.read_byte()?;

		let version = version_and_type & VERSION_MASK;
		let xt_type = version_and_type & TYPE_MASK;

		let preamble = match (version, xt_type) {
			(extrinsic_version @ LEGACY_EXTRINSIC_FORMAT_VERSION..=EXTRINSIC_FORMAT_VERSION, BARE_EXTRINSIC) => {
				Self::Bare(extrinsic_version)
			},
			(LEGACY_EXTRINSIC_FORMAT_VERSION, SIGNED_EXTRINSIC) => {
				let address = MultiAddress::decode(input)?;
				let signature = MultiSignature::decode(input)?;
				let ext = Extension::decode(input)?;
				Self::Signed(address, signature, ext)
			},
			(EXTRINSIC_FORMAT_VERSION, GENERAL_EXTRINSIC) => {
				let ext_version = ExtensionVersion::decode(input)?;
				let ext = Extension::decode(input)?;
				Self::General(ext_version, ext)
			},
			(_, _) => return Err("Invalid transaction version".into()),
		};

		Ok(preamble)
	}
}

impl Encode for Preamble {
	fn size_hint(&self) -> usize {
		match &self {
			Preamble::Bare(_) => EXTRINSIC_FORMAT_VERSION.size_hint(),
			Preamble::Signed(address, signature, ext) => LEGACY_EXTRINSIC_FORMAT_VERSION
				.size_hint()
				.saturating_add(address.size_hint())
				.saturating_add(signature.size_hint())
				.saturating_add(ext.size_hint()),
			Preamble::General(ext_version, ext) => EXTRINSIC_FORMAT_VERSION
				.size_hint()
				.saturating_add(ext_version.size_hint())
				.saturating_add(ext.size_hint()),
		}
	}

	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		match &self {
			Preamble::Bare(extrinsic_version) => {
				(extrinsic_version | BARE_EXTRINSIC).encode_to(dest);
			},
			Preamble::Signed(address, signature, ext) => {
				(LEGACY_EXTRINSIC_FORMAT_VERSION | SIGNED_EXTRINSIC).encode_to(dest);
				address.encode_to(dest);
				signature.encode_to(dest);
				ext.encode_to(dest);
			},
			Preamble::General(ext_version, ext) => {
				(EXTRINSIC_FORMAT_VERSION | GENERAL_EXTRINSIC).encode_to(dest);
				ext_version.encode_to(dest);
				ext.encode_to(dest);
			},
		}
	}
}

impl Preamble {
	/// Returns `Some` if this is a signed extrinsic, together with the relevant inner fields.
	pub fn to_signed(self) -> Option<(MultiAddress, MultiSignature, Extension)> {
		match self {
			Self::Signed(a, s, e) => Some((a, s, e)),
			_ => None,
		}
	}

	/// Returns `Some` if this is a signed extrinsic, together with the relevant inner fields.
	pub fn to_signed_ref(&self) -> Option<(&MultiAddress, &MultiSignature, &Extension)> {
		match self {
			Self::Signed(a, s, e) => Some((a, s, e)),
			_ => None,
		}
	}
}

impl std::fmt::Debug for Preamble {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Bare(_) => write!(f, "Bare"),
			Self::Signed(address, _, tx_ext) => write!(f, "Signed({:?}, {:?})", address, tx_ext),
			Self::General(ext_version, tx_ext) => write!(f, "General({:?}, {:?})", ext_version, tx_ext),
		}
	}
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Extension {
	pub era: Era,
	#[codec(compact)]
	pub nonce: u32,
	#[codec(compact)]
	pub tip: u128,
}

#[derive(Debug, Clone)]
pub struct ExtrinsicCall(pub Vec<u8>);

impl ExtrinsicCall {
	pub fn new(call: Vec<u8>) -> Self {
		Self(call)
	}

	pub fn from_parts(pallet_id: u8, variant_id: u8, data: Vec<u8>) -> Self {
		let mut tmp = Vec::with_capacity(2 + data.len());
		tmp.push(pallet_id);
		tmp.push(variant_id);
		tmp.extend(data);
		Self(tmp)
	}

	pub fn as_slice(&self) -> &[u8] {
		&self.0
	}

	pub fn hash(&self) -> [u8; 32] {
		sp_crypto_hashing::blake2_256(&self.0)
	}
}

impl Encode for ExtrinsicCall {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		dest.write(&self.0);
	}
}

impl Decode for ExtrinsicCall {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let call = decode_already_decoded(input)?;
		Ok(Self(call))
	}
}

impl<T: HasHeader + Encode> From<&T> for ExtrinsicCall {
	fn from(value: &T) -> Self {
		let mut call = Vec::with_capacity(2);
		call.push(T::HEADER_INDEX.0);
		call.push(T::HEADER_INDEX.1);
		call.extend(value.encode());
		Self(call)
	}
}

impl From<Vec<u8>> for ExtrinsicCall {
	fn from(value: Vec<u8>) -> Self {
		Self(value)
	}
}

impl TryFrom<String> for ExtrinsicCall {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&str> for ExtrinsicCall {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let call = const_hex::decode(value.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		Ok(Self(call))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ExtrinsicCallBorrowed<'a>(pub &'a [u8]);

impl<'a> ExtrinsicCallBorrowed<'a> {
	pub fn new(call: &'a [u8]) -> Self {
		Self(call)
	}

	pub fn as_slice(&self) -> &[u8] {
		self.0
	}

	pub fn hash(&self) -> [u8; 32] {
		sp_crypto_hashing::blake2_256(self.0)
	}
}

impl<'a> Encode for ExtrinsicCallBorrowed<'a> {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		dest.write(self.0);
	}
}

fn encode_extrinsic(preamble: &Preamble, call: &[u8]) -> Vec<u8> {
	let mut tmp = preamble.encode();
	tmp.extend_from_slice(call);

	let compact_len = codec::Compact::<u32>(tmp.len() as u32);
	// Allocate the output buffer with the correct length
	let mut output = Vec::with_capacity(compact_len.size_hint() + tmp.len());
	compact_len.encode_to(&mut output);
	output.extend(tmp);

	output
}

#[derive(Debug, Clone)]
pub struct Extrinsic {
	pub preamble: Preamble,
	pub call: ExtrinsicCall,
}

impl Extrinsic {
	pub fn new(preamble: Preamble, call: ExtrinsicCall) -> Self {
		Self { preamble, call }
	}

	pub fn new_signed(account_id: AccountId, signature: [u8; 64], extension: Extension, call: ExtrinsicCall) -> Self {
		let address = MultiAddress::Id(account_id);
		let signature = MultiSignature::Sr25519(signature);
		let preamble = Preamble::Signed(address, signature, extension);

		Self { preamble, call }
	}

	pub fn new_general(extension_version: ExtensionVersion, extension: Extension, call: ExtrinsicCall) -> Self {
		let preamble = Preamble::General(extension_version, extension);

		Self { preamble, call }
	}

	pub fn new_bare(extrinsic_version: ExtrinsicVersion, call: ExtrinsicCall) -> Self {
		let preamble = Preamble::Bare(extrinsic_version);

		Self { preamble, call }
	}

	pub fn hash(&self) -> H256 {
		let encoded = self.encode();
		BlakeTwo256.hash(&encoded)
	}
}

impl TryFrom<&str> for Extrinsic {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let call = const_hex::decode(value.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		Self::try_from(call.as_slice()).map_err(|e| e.to_string())
	}
}

impl TryFrom<&Vec<u8>> for Extrinsic {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for Extrinsic {
	type Error = codec::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut value = value;
		Self::decode(&mut value)
	}
}

impl Encode for Extrinsic {
	fn encode(&self) -> Vec<u8> {
		encode_extrinsic(&self.preamble, &self.call.0)
	}
}

impl Decode for Extrinsic {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		// This is a little more complicated than usual since the binary format must be compatible
		// with SCALE's generic `Vec<u8>` type. Basically this just means accepting that there
		// will be a prefix of vector length.
		let expected_length: Compact<u32> = Decode::decode(input)?;
		let expected_length = expected_length.0 as usize;
		let mut input = CountedInput::new(input);

		let preamble: Preamble = Decode::decode(&mut input)?;

		let call_length = expected_length
			.checked_sub(input.count() as usize)
			.ok_or("Preamble bytes exceed expected extrinsic length")?;
		let mut call = vec![0u8; call_length];
		input.read(&mut call)?;

		if input.count() as usize != expected_length {
			return Err("Invalid length prefix".into());
		}

		Ok(Self { preamble, call: ExtrinsicCall::new(call) })
	}
}

impl Serialize for Extrinsic {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = self.encode();
		impl_serde::serialize::serialize(&bytes, serializer)
	}
}

impl<'de> Deserialize<'de> for Extrinsic {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..]).map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}

#[derive(Debug, Clone)]
pub struct ExtrinsicBorrowed<'a> {
	pub preamble: Preamble,
	pub call: ExtrinsicCallBorrowed<'a>,
}

impl<'a> ExtrinsicBorrowed<'a> {
	pub fn new(preamble: Preamble, call: &'a [u8]) -> Self {
		Self { preamble, call: ExtrinsicCallBorrowed::new(call) }
	}

	pub fn new_signed(account_id: AccountId, signature: [u8; 64], extension: Extension, call: &'a [u8]) -> Self {
		let address = MultiAddress::Id(account_id);
		let signature = MultiSignature::Sr25519(signature);
		let preamble = Preamble::Signed(address, signature, extension);

		Self { preamble, call: ExtrinsicCallBorrowed::new(call) }
	}

	pub fn new_general(extension_version: ExtensionVersion, extension: Extension, call: &'a [u8]) -> Self {
		let preamble = Preamble::General(extension_version, extension);

		Self { preamble, call: ExtrinsicCallBorrowed::new(call) }
	}

	pub fn new_bare(extrinsic_version: ExtrinsicVersion, call: &'a [u8]) -> Self {
		let preamble = Preamble::Bare(extrinsic_version);

		Self { preamble, call: ExtrinsicCallBorrowed::new(call) }
	}

	pub fn hash(&self) -> H256 {
		let encoded = self.encode();
		BlakeTwo256.hash(&encoded)
	}
}

impl<'a> Encode for ExtrinsicBorrowed<'a> {
	fn encode(&self) -> Vec<u8> {
		encode_extrinsic(&self.preamble, self.call.0)
	}
}

#[derive(Debug, Clone)]
pub struct ExtensionImplicit {
	pub spec_version: u32,
	pub tx_version: u32,
	pub genesis_hash: H256,
	pub fork_hash: H256,
}
impl Encode for ExtensionImplicit {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.spec_version.encode_to(dest);
		self.tx_version.encode_to(dest);
		self.genesis_hash.encode_to(dest);
		self.fork_hash.encode_to(dest);
	}
}
impl Decode for ExtensionImplicit {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let spec_version = Decode::decode(input)?;
		let tx_version = Decode::decode(input)?;
		let genesis_hash = Decode::decode(input)?;
		let fork_hash = Decode::decode(input)?;
		Ok(Self { spec_version, tx_version, genesis_hash, fork_hash })
	}
}

// There is no need for Encode and Decode
#[derive(Debug, Clone)]
pub struct SignedPayload<'a> {
	/// Already encoded call
	pub call: &'a [u8],
	pub extension: &'a Extension,
	pub implicit: &'a ExtensionImplicit,
}

impl<'a> SignedPayload<'a> {
	pub fn new(call: &'a [u8], extension: &'a Extension, implicit: &'a ExtensionImplicit) -> Self {
		Self { call, extension, implicit }
	}

	pub fn sign_static(
		call: &'a [u8],
		extension: &'a Extension,
		implicit: &'a ExtensionImplicit,
		signer: &Keypair,
	) -> [u8; 64] {
		Self { call, extension, implicit }.sign(signer)
	}

	pub fn sign(&self, signer: &Keypair) -> [u8; 64] {
		let size_hint = self.call.size_hint() + self.extension.size_hint() + self.implicit.size_hint();

		let mut data: Vec<u8> = Vec::with_capacity(size_hint);
		data.extend(self.call);
		self.extension.encode_to(&mut data);
		self.implicit.encode_to(&mut data);

		if data.len() > 256 {
			let hash = BlakeTwo256.hash(&data);
			signer.sign(hash.as_ref()).0
		} else {
			signer.sign(&data).0
		}
	}
}
