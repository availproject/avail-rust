use crate::{Error, rpc};
use codec::{Decode, Encode};
use primitive_types::H256;
use std::marker::PhantomData;
use subxt_rpcs::RpcClient;

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
		std::format!("0x{}", const_hex::encode(Self::encode_storage_key()))
	}

	/// Decodes the Hex and SCALE encoded Storage Value
	/// This is equal to Hex::decode + Self::decode
	///
	/// If you need to decode bytes call `decode`
	fn hex_decode(value: &str) -> Result<Self::VALUE, codec::Error> {
		let Ok(hex_decoded) = const_hex::decode(value.trim_start_matches("0x")) else {
			return Err("Failed to hex decode storage".into());
		};
		Self::decode(&mut hex_decoded.as_slice())
	}

	/// Decodes the SCALE encoded Storage Value
	///
	/// If you need to decode Hex string call `hex_decode`
	fn decode(value: &mut &[u8]) -> Result<Self::VALUE, codec::Error> {
		Self::VALUE::decode(value)
	}

	/// Fetches and decodes a Storage Value
	///
	/// Returns None if no Storage Value is present
	fn fetch(
		client: &RpcClient,
		at: Option<H256>,
	) -> impl std::future::Future<Output = Result<Option<Self::VALUE>, Error>> {
		async move {
			let storage_key = const_hex::encode(Self::encode_storage_key());

			let storage_value = rpc::state::get_storage(client, &storage_key, at).await?;
			let Some(storage_value) = storage_value else {
				return Ok(None);
			};

			let storage_value = Self::decode(&mut storage_value.as_slice())?;
			Ok(Some(storage_value))
		}
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
		std::format!("0x{}", const_hex::encode(Self::encode_partial_key()))
	}

	fn encode_storage_key(key: &Self::KEY) -> Vec<u8> {
		let mut storage_key: Vec<u8> = Vec::new();
		storage_key.extend_from_slice(&Self::encode_partial_key());

		let encoded_key = key.encode();
		storage_key.extend_from_slice(&Self::KEY_HASHER.hash(&encoded_key));

		storage_key
	}

	fn hex_encode_storage_key(key: &Self::KEY) -> String {
		std::format!("0x{}", const_hex::encode(Self::encode_storage_key(key)))
	}

	/// Decodes the Hex and SCALE encoded Storage Key
	/// This is equal to Hex::decode + Self::decode_storage_key
	///
	/// If you need to decode bytes call `decode_storage_key`
	#[inline(always)]
	fn decode_hex_storage_key(value: &str) -> Result<Self::KEY, codec::Error> {
		let Ok(hex_decoded) = const_hex::decode(value.trim_start_matches("0x")) else {
			return Err("Failed to hex decode storage key".into());
		};
		Self::decode_storage_key(&mut hex_decoded.as_slice())
	}

	/// Decodes the SCALE encoded Storage Key
	///
	/// If you need to decode Hex string call `decode_hex_storage_key`
	fn decode_storage_key(value: &mut &[u8]) -> Result<Self::KEY, codec::Error> {
		if value.len() < 32 {
			return Err("Storage Key is malformed. Has less than 32 bytes".into());
		}

		// Skip pallet/variant
		*value = &value[32..];

		Self::KEY_HASHER.from_hash::<Self::KEY>(value)
	}

	/// Decodes the Hex and SCALE encoded Storage Value
	/// This is equal to Hex::decode + Self::decode_storage_value
	///
	/// If you need to decode bytes call `decode_storage_value`
	#[inline(always)]
	fn decode_hex_storage_value(value: &str) -> Result<Self::VALUE, codec::Error> {
		let Ok(hex_decoded) = const_hex::decode(value.trim_start_matches("0x")) else {
			return Err("Failed to hex decode storage value".into());
		};
		Self::decode_storage_value(&mut hex_decoded.as_slice())
	}

	/// Decodes the SCALE encoded Storage Value
	///
	/// If you need to decode Hex string call `decode_hex_storage_value`
	fn decode_storage_value(value: &mut &[u8]) -> Result<Self::VALUE, codec::Error> {
		Self::VALUE::decode(value)
	}

	/// Fetches and decodes a Storage Value
	///
	/// Returns None if no Storage Value is present
	fn fetch(
		client: &RpcClient,
		key: &Self::KEY,
		at: Option<H256>,
	) -> impl std::future::Future<Output = Result<Option<Self::VALUE>, Error>> {
		async move {
			let storage_key = const_hex::encode(Self::encode_storage_key(key));
			let storage_value = rpc::state::get_storage(client, &storage_key, at).await?;
			let Some(storage_value) = storage_value else {
				return Ok(None);
			};

			let storage_value = Self::decode_storage_value(&mut storage_value.as_slice())?;
			Ok(Some(storage_value))
		}
	}

	fn iter(client: RpcClient, block_hash: H256) -> StorageMapIterator<Self>
	where
		Self: Sized,
	{
		StorageMapIterator::new(client, block_hash)
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

	fn encode_partial_key(key1: &Self::KEY1) -> Vec<u8> {
		use sp_crypto_hashing::twox_128;

		let mut encoded_storage_key = Vec::new();
		encoded_storage_key.extend_from_slice(&twox_128(Self::PALLET_NAME.as_bytes()));
		encoded_storage_key.extend_from_slice(&twox_128(Self::STORAGE_NAME.as_bytes()));
		encoded_storage_key.extend_from_slice(&Self::KEY1_HASHER.hash(&key1.encode()));

		encoded_storage_key
	}

	fn hex_encode_partial_key(key1: &Self::KEY1) -> String {
		std::format!("0x{}", const_hex::encode(Self::encode_partial_key(key1)))
	}

	fn encode_storage_key(key1: &Self::KEY1, key2: &Self::KEY2) -> Vec<u8> {
		let mut storage_key: Vec<u8> = Vec::new();
		storage_key.extend_from_slice(&Self::encode_partial_key(key1));
		storage_key.extend_from_slice(&Self::KEY2_HASHER.hash(&key2.encode()));

		storage_key
	}

	fn hex_encode_storage_key(key1: &Self::KEY1, key2: &Self::KEY2) -> String {
		std::format!("0x{}", const_hex::encode(Self::encode_storage_key(key1, key2)))
	}

	fn decode_partial_key(value: &mut &[u8]) -> Result<Self::KEY1, codec::Error> {
		if value.len() < 32 {
			return Err("Storage Key is malformed. Has less than 32 bytes".into());
		}

		// Skip pallet/variant
		*value = &value[32..];

		Self::KEY1_HASHER.from_hash::<Self::KEY1>(value)
	}

	/// Decodes the Hex and SCALE encoded Storage Key
	/// This is equal to Hex::decode + Self::decode_storage_key
	///
	/// If you need to decode bytes call `decode_storage_key`
	fn decode_hex_storage_key(value: &str) -> Result<(Self::KEY1, Self::KEY2), codec::Error> {
		let Ok(hex_decoded) = const_hex::decode(value.trim_start_matches("0x")) else {
			return Err("Failed to hex decode storage key".into());
		};
		Self::decode_storage_key(&mut hex_decoded.as_slice())
	}

	/// Decodes the SCALE encoded Storage Key
	///
	/// If you need to decode Hex string call `decode_hex_storage_key`
	fn decode_storage_key(value: &mut &[u8]) -> Result<(Self::KEY1, Self::KEY2), codec::Error> {
		if value.len() < 32 {
			return Err("Storage Key is malformed. Has less than 32 bytes".into());
		}

		// Skip pallet/variant
		*value = &value[32..];

		let key1 = Self::KEY1_HASHER.from_hash::<Self::KEY1>(value)?;
		let key2 = Self::KEY2_HASHER.from_hash::<Self::KEY2>(value)?;
		Ok((key1, key2))
	}

	/// Decodes the Hex and SCALE encoded Storage Value
	/// This is equal to Hex::decode + Self::decode_storage_value
	///
	/// If you need to decode bytes call `decode_storage_value`
	fn decode_hex_storage_value(value: &str) -> Result<Self::VALUE, codec::Error> {
		let Ok(hex_decoded) = const_hex::decode(value.trim_start_matches("0x")) else {
			return Err("Failed to hex decode storage value".into());
		};
		Self::decode_storage_value(&mut hex_decoded.as_slice())
	}

	/// Decodes the SCALE encoded Storage Value
	///
	/// If you need to decode Hex string call `decode_hex_storage_value`
	fn decode_storage_value(value: &mut &[u8]) -> Result<Self::VALUE, codec::Error> {
		Self::VALUE::decode(value)
	}

	/// Fetches and decodes a Storage Value
	///
	/// Returns None if no Storage Value is present
	fn fetch(
		client: &RpcClient,
		key_1: &Self::KEY1,
		key_2: &Self::KEY2,
		at: Option<H256>,
	) -> impl std::future::Future<Output = Result<Option<Self::VALUE>, Error>> {
		async move {
			let storage_key = const_hex::encode(Self::encode_storage_key(key_1, key_2));
			let storage_value = rpc::state::get_storage(client, &storage_key, at).await?;
			let Some(storage_value) = storage_value else {
				return Ok(None);
			};

			let storage_value = Self::decode_storage_value(&mut storage_value.as_slice())?;
			Ok(Some(storage_value))
		}
	}

	fn iter(client: RpcClient, key_1: &Self::KEY1, block_hash: H256) -> StorageDoubleMapIterator<Self>
	where
		Self: Sized,
	{
		StorageDoubleMapIterator::new(client, key_1, block_hash)
	}
}

#[derive(Clone)]
pub struct StorageMapIterator<T: StorageMap> {
	client: RpcClient,
	phantom: PhantomData<T>,
	block_hash: H256,
	fetched_keys: Vec<String>,
	last_key: Option<String>,
	is_done: bool,
	prefix: String,
}

impl<T: StorageMap> StorageMapIterator<T> {
	pub fn new(client: RpcClient, block_hash: H256) -> Self {
		Self {
			client,
			phantom: PhantomData::<T>,
			block_hash,
			fetched_keys: Vec::new(),
			last_key: None,
			is_done: false,
			prefix: const_hex::encode(T::encode_partial_key()),
		}
	}

	pub async fn next_key_value(&mut self) -> Result<Option<(T::KEY, T::VALUE)>, Error> {
		if self.is_done {
			return Ok(None);
		}

		// Fetch new keys
		if self.fetched_keys.is_empty() {
			self.fetch_new_keys().await?;
		}

		let Some(storage_key) = self.fetched_keys.last() else {
			return Ok(None);
		};

		let Some(storage_value) = self.fetch_storage_value(storage_key).await? else {
			return Ok(None);
		};

		let key = const_hex::decode(storage_key.trim_start_matches("0x")).map_err(|x| x.to_string())?;
		let key = T::decode_storage_key(&mut key.as_slice())?;

		self.last_key = Some(storage_key.clone());
		self.fetched_keys.pop();

		Ok(Some((key, storage_value)))
	}

	pub async fn next(&mut self) -> Result<Option<T::VALUE>, Error> {
		if self.is_done {
			return Ok(None);
		}

		// Fetch new keys
		if self.fetched_keys.is_empty() {
			self.fetch_new_keys().await?;
		}

		let Some(storage_key) = self.fetched_keys.last() else {
			return Ok(None);
		};

		let Some(storage_value) = self.fetch_storage_value(storage_key).await? else {
			return Ok(None);
		};

		self.last_key = Some(storage_key.clone());
		self.fetched_keys.pop();

		Ok(Some(storage_value))
	}

	async fn fetch_new_keys(&mut self) -> Result<(), Error> {
		self.fetched_keys = rpc::state::get_keys_paged(
			&self.client,
			Some(self.prefix.clone()),
			100,
			self.last_key.clone(),
			Some(self.block_hash),
		)
		.await?;

		self.fetched_keys.reverse();
		if self.fetched_keys.is_empty() {
			self.is_done = true
		}

		Ok(())
	}

	async fn fetch_storage_value(&self, key: &str) -> Result<Option<T::VALUE>, Error> {
		let storage_value = rpc::state::get_storage(&self.client, key, Some(self.block_hash)).await?;
		let Some(storage_value) = storage_value else {
			return Ok(None);
		};
		let storage_value = T::decode_storage_value(&mut storage_value.as_slice())?;

		Ok(Some(storage_value))
	}
}

#[derive(Clone)]
pub struct StorageDoubleMapIterator<T: StorageDoubleMap> {
	client: RpcClient,
	phantom: PhantomData<T>,
	block_hash: H256,
	fetched_keys: Vec<String>,
	last_key: Option<String>,
	is_done: bool,
	prefix: String,
}

impl<T: StorageDoubleMap> StorageDoubleMapIterator<T> {
	pub fn new(client: RpcClient, key_1: &T::KEY1, block_hash: H256) -> Self {
		Self {
			client,
			phantom: PhantomData::<T>,
			block_hash,
			fetched_keys: Vec::new(),
			last_key: None,
			is_done: false,

			prefix: const_hex::encode(T::encode_partial_key(key_1)),
		}
	}

	pub async fn next_key_value(&mut self) -> Result<Option<(T::KEY1, T::KEY2, T::VALUE)>, Error> {
		if self.is_done {
			return Ok(None);
		}

		// Fetch new keys
		if self.fetched_keys.is_empty() {
			self.fetch_new_keys().await?;
		}

		let Some(storage_key) = self.fetched_keys.last() else {
			return Ok(None);
		};

		let Some(storage_value) = self.fetch_storage_value(storage_key).await? else {
			return Ok(None);
		};

		let key = const_hex::decode(storage_key.trim_start_matches("0x")).map_err(|x| x.to_string())?;
		let (key1, key2) = T::decode_storage_key(&mut key.as_slice())?;

		self.last_key = Some(storage_key.clone());
		self.fetched_keys.pop();

		Ok(Some((key1, key2, storage_value)))
	}

	pub async fn next(&mut self) -> Result<Option<T::VALUE>, Error> {
		if self.is_done {
			return Ok(None);
		}

		// Fetch new keys
		if self.fetched_keys.is_empty() {
			self.fetch_new_keys().await?;
		}

		let Some(storage_key) = self.fetched_keys.last() else {
			return Ok(None);
		};

		let Some(storage_value) = self.fetch_storage_value(storage_key).await? else {
			return Ok(None);
		};

		self.last_key = Some(storage_key.clone());
		self.fetched_keys.pop();

		Ok(Some(storage_value))
	}

	async fn fetch_new_keys(&mut self) -> Result<(), Error> {
		self.fetched_keys = rpc::state::get_keys_paged(
			&self.client,
			Some(self.prefix.clone()),
			100,
			self.last_key.clone(),
			Some(self.block_hash),
		)
		.await?;

		self.fetched_keys.reverse();
		if self.fetched_keys.is_empty() {
			self.is_done = true
		}

		Ok(())
	}

	async fn fetch_storage_value(&self, key: &str) -> Result<Option<T::VALUE>, Error> {
		let storage_value = rpc::state::get_storage(&self.client, key, Some(self.block_hash)).await?;
		let Some(storage_value) = storage_value else {
			return Ok(None);
		};
		let storage_value = T::decode_storage_value(&mut storage_value.as_slice())?;

		Ok(Some(storage_value))
	}
}
