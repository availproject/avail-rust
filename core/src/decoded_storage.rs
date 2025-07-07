use codec::{Decode, Encode};

#[derive(Debug, Clone, Copy)]
pub enum StorageHasher {
	/// 128-bit Blake2 hash.
	Blake2_128,
	/// 256-bit Blake2 hash.
	Blake2_256,
	/// Multiple 128-bit Blake2 hashes concatenated.
	Blake2_128Concat,
	/// 128-bit XX hash.
	Twox128,
	/// 256-bit XX hash.
	Twox256,
	/// Multiple 64-bit XX hashes concatenated.
	Twox64Concat,
	/// Identity hashing (no hashing).
	Identity,
}

impl StorageHasher {
	pub fn hash(&self, data: &[u8]) -> Vec<u8> {
		match self {
			StorageHasher::Blake2_128 => sp_crypto_hashing::blake2_128(data).into(),
			StorageHasher::Blake2_256 => sp_crypto_hashing::blake2_256(data).into(),
			StorageHasher::Blake2_128Concat => {
				let mut hash = sp_crypto_hashing::blake2_128(data).to_vec();
				hash.extend_from_slice(data);
				hash
			},
			StorageHasher::Twox128 => sp_crypto_hashing::twox_128(data).into(),
			StorageHasher::Twox256 => sp_crypto_hashing::twox_256(data).into(),
			StorageHasher::Twox64Concat => {
				let mut hash = sp_crypto_hashing::twox_64(data).to_vec();
				hash.extend_from_slice(data);
				hash
			},
			StorageHasher::Identity => data.to_vec(),
		}
	}

	pub fn from_hash<Key: codec::Decode>(&self, data: &mut &[u8]) -> Result<Key, codec::Error> {
		match self {
			StorageHasher::Blake2_128Concat => {
				if data.len() < 17 {
					return Err(codec::Error::from("Not enough data to compute Blake2_128Concat"));
				}
				Key::decode(&mut &data[16..])
			},
			StorageHasher::Twox64Concat => {
				if data.len() < 9 {
					return Err(codec::Error::from("Not enough data to compute Twox64Concat"));
				}
				Key::decode(&mut &data[8..])
			},
			StorageHasher::Identity => Key::decode(data),
			_ => unimplemented!(),
		}
	}
}

pub trait StorageValue {
	const PALLET_NAME: &str;
	const STORAGE_NAME: &str;
	type VALUE: codec::Decode;

	fn encode_storage_key() -> [u8; 32] {
		use sp_crypto_hashing::twox_128;

		let mut encoded_storage_key = [0u8; 32];
		encoded_storage_key[0..16].copy_from_slice(&twox_128(Self::PALLET_NAME.as_bytes()));
		encoded_storage_key[16..].copy_from_slice(&twox_128(Self::STORAGE_NAME.as_bytes()));

		encoded_storage_key
	}

	fn hex_encode_storage_key() -> String {
		std::format!("0x{}", hex::encode(&Self::encode_storage_key()))
	}

	fn decode(value: &mut &[u8]) -> Result<Self::VALUE, codec::Error> {
		Self::VALUE::decode(value)
	}
}

pub trait StorageMap {
	const PALLET_NAME: &str;
	const STORAGE_NAME: &str;
	const KEY_HASHER: StorageHasher;
	type KEY: codec::Decode + codec::Encode;
	type VALUE: codec::Decode;

	fn encode_partial_key() -> [u8; 32] {
		use sp_crypto_hashing::twox_128;

		let mut encoded_storage_key = [0u8; 32];
		encoded_storage_key[0..16].copy_from_slice(&twox_128(Self::PALLET_NAME.as_bytes()));
		encoded_storage_key[16..].copy_from_slice(&twox_128(Self::STORAGE_NAME.as_bytes()));

		encoded_storage_key
	}

	fn hex_encode_partial_key() -> String {
		std::format!("0x{}", hex::encode(&Self::encode_partial_key()))
	}

	fn encode_storage_key(key: Self::KEY) -> Vec<u8> {
		let mut storage_key: Vec<u8> = Vec::new();
		storage_key.extend_from_slice(&Self::encode_partial_key());

		let encoded_key = key.encode();
		storage_key.extend_from_slice(&Self::KEY_HASHER.hash(&encoded_key));

		storage_key
	}

	fn hex_encode_storage_key(key: Self::KEY) -> String {
		std::format!("0x{}", hex::encode(&Self::encode_storage_key(key)))
	}

	fn decode_storage_key(value: &mut &[u8]) -> Result<Self::KEY, codec::Error> {
		// Skip pallet/variant
		*value = &value[32..];

		Self::KEY_HASHER.from_hash::<Self::KEY>(value)
	}

	fn decode_storage_value(value: &mut &[u8]) -> Result<Self::VALUE, codec::Error> {
		Self::VALUE::decode(value)
	}
}

pub trait StorageDoubleMap {
	const PALLET_NAME: &str;
	const STORAGE_NAME: &str;
	const KEY1_HASHER: StorageHasher;
	const KEY2_HASHER: StorageHasher;
	type KEY1: codec::Decode + codec::Encode;
	type KEY2: codec::Decode + codec::Encode;
	type VALUE: codec::Decode;

	fn encode_partial_key(key1: Self::KEY1) -> Vec<u8> {
		use sp_crypto_hashing::twox_128;

		let mut encoded_storage_key = Vec::new();
		encoded_storage_key.extend_from_slice(&twox_128(Self::PALLET_NAME.as_bytes()));
		encoded_storage_key.extend_from_slice(&twox_128(Self::STORAGE_NAME.as_bytes()));
		encoded_storage_key.extend_from_slice(&Self::KEY1_HASHER.hash(&key1.encode()));

		encoded_storage_key
	}

	fn hex_encode_partial_key(key1: Self::KEY1) -> String {
		std::format!("0x{}", hex::encode(&Self::encode_partial_key(key1)))
	}

	fn encode_storage_key(key1: Self::KEY1, key2: Self::KEY2) -> Vec<u8> {
		let mut storage_key: Vec<u8> = Vec::new();
		storage_key.extend_from_slice(&Self::encode_partial_key(key1));
		storage_key.extend_from_slice(&Self::KEY2_HASHER.hash(&key2.encode()));

		storage_key
	}

	fn hex_encode_storage_key(key1: Self::KEY1, key2: Self::KEY2) -> String {
		std::format!("0x{}", hex::encode(&Self::encode_storage_key(key1, key2)))
	}

	fn decode_partial_key(value: &mut &[u8]) -> Result<Self::KEY1, codec::Error> {
		// Skip pallet/variant
		*value = &value[32..];

		Self::KEY1_HASHER.from_hash::<Self::KEY1>(value)
	}

	fn decode_storage_key(value: &mut &[u8]) -> Result<(Self::KEY1, Self::KEY2), codec::Error> {
		// Skip pallet/variant
		*value = &value[32..];

		let key1 = Self::KEY1_HASHER.from_hash::<Self::KEY1>(value)?;
		let key2 = Self::KEY2_HASHER.from_hash::<Self::KEY2>(value)?;
		Ok((key1, key2))
	}

	fn decode_storage_value(value: &mut &[u8]) -> Result<Self::VALUE, codec::Error> {
		Self::VALUE::decode(value)
	}
}

/* pub struct DataAvailabilityAppKeys;

#[derive(Debug, Clone, codec::Decode)]
pub struct AppKey {
	pub owner: AccountId,
	#[codec(compact)]
	pub id: u32,
}

impl StorageMap for DataAvailabilityAppKeys {
	const PALLET_NAME: &str = "DataAvailability";
	const STORAGE_NAME: &str = "AppKeys";
	const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
	type KEY = Vec<u8>;
	type VALUE = AppKey;
}

pub struct TimestampNow;

impl StorageValue for TimestampNow {
	const PALLET_NAME: &str = "Timestamp";
	const STORAGE_NAME: &str = "Now";
	type VALUE = u64;
}

pub struct StakingErasValidatorPrefs;

impl StorageDoubleMap for StakingErasValidatorPrefs {
	const PALLET_NAME: &str = "Staking";
	const STORAGE_NAME: &str = "ErasValidatorPrefs";
	const KEY1_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const KEY2_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	type KEY1 = u32;
	type KEY2 = AccountId;
	type VALUE = ValidatorPrefs;
}

#[derive(Debug, Clone, codec::Encode, codec::Decode)]
pub struct ValidatorPrefs {
	/// Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	#[codec(compact)]
	pub commission: u32,
	/// Whether or not this validator is accepting more nominations. If `true`, then no nominator
	/// who is not already nominating this validator may nominate them. By default, validators
	/// are accepting nominations.
	pub blocked: bool,
}
 */
