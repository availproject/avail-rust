use crate::{
	clients::Client,
	subxt_core::{self, storage::address::Address, utils::Yes},
};
use avail_rust_core::{H256, StorageMap, StorageValue};

#[cfg(feature = "subxt")]
use crate::subxt::Error;
#[cfg(feature = "subxt")]
use crate::subxt::backend::StreamOfResults;
#[cfg(feature = "subxt")]
use crate::subxt::storage::StorageKeyValuePair;
#[cfg(feature = "subxt")]
use std::future::Future;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct StorageClient {
	client: Client,
}

impl StorageClient {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	#[cfg(feature = "subxt")]
	pub fn iter<Addr>(
		&self,
		address: Addr,
		at: H256,
	) -> impl Future<Output = Result<StreamOfResults<StorageKeyValuePair<Addr>>, Error>> + 'static
	where
		Addr: Address<IsIterable = Yes> + 'static,
		Addr::Keys: 'static + Sized,
	{
		let storage_client = self.client.subxt_storage_client();
		let storage = storage_client.at(at);
		storage.iter(address)
	}

	pub async fn fetch<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Option<Addr::Target>, avail_rust_core::Error>
	where
		Addr: Address<IsFetchable = Yes> + 'address,
	{
		#[cfg(feature = "subxt")]
		{
			let storage = self.client.subxt_storage_client().at(at);
			Ok(storage.fetch(address).await?)
		}

		#[cfg(not(feature = "subxt"))]
		{
			let metadata = self.client.online_client().metadata();
			let key = subxt_core::storage::get_address_bytes(address, &metadata)?;
			let key = std::format!("0x{}", hex::encode(key));
			if let Some(data) = self.client.rpc_api().state_get_storage(&key, Some(at)).await? {
				let val = subxt_core::storage::decode_value(&mut &*data, address, &metadata)?;
				Ok(Some(val))
			} else {
				Ok(None)
			}
		}
	}

	pub async fn fetch_or_default<'address, Addr>(
		&self,
		address: &Addr,
		at: H256,
	) -> Result<Addr::Target, avail_rust_core::Error>
	where
		Addr: Address<IsFetchable = Yes, IsDefaultable = Yes> + 'address,
	{
		#[cfg(feature = "subxt")]
		{
			let storage = self.client.subxt_storage_client().at(at);
			Ok(storage.fetch_or_default(address).await?)
		}

		#[cfg(not(feature = "subxt"))]
		{
			if let Some(data) = self.fetch(address, at).await? {
				Ok(data)
			} else {
				let metadata = self.client.online_client().metadata();
				let val = subxt_core::storage::default_value(address, &metadata)?;
				Ok(val)
			}
		}
	}

	// constants
	pub async fn constants_at<'address, Addr>(&self, address: &Addr) -> Result<Addr::Target, avail_rust_core::Error>
	where
		Addr: subxt_core::constants::address::Address,
	{
		let metadata = self.client.online_client().metadata();
		let val = subxt_core::constants::get(address, &metadata)?;
		Ok(val)
	}
}

#[derive(Clone)]
pub struct StorageValueFetcher<T: StorageValue> {
	phantom: PhantomData<T>,
}

impl<T: StorageValue> StorageValueFetcher<T> {
	pub async fn fetch(client: &Client, at: Option<H256>) -> Result<Option<T::VALUE>, avail_rust_core::Error> {
		let storage_key = hex::encode(T::encode_storage_key());
		let storage_value = client.rpc_api().state_get_storage(&storage_key, at).await?;
		let Some(storage_value) = storage_value else {
			return Ok(None);
		};

		let storage_value = T::decode(&mut storage_value.as_slice())?;
		return Ok(Some(storage_value));
	}
}

#[derive(Clone)]
pub struct StorageMapFetcher<T: StorageMap> {
	phantom: PhantomData<T>,
}

impl<T: StorageMap> StorageMapFetcher<T> {
	pub async fn fetch(
		client: &Client,
		key: T::KEY,
		at: Option<H256>,
	) -> Result<Option<T::VALUE>, avail_rust_core::Error> {
		let storage_key = hex::encode(T::encode_storage_key(key));
		let storage_value = client.rpc_api().state_get_storage(&storage_key, at).await?;
		let Some(storage_value) = storage_value else {
			return Ok(None);
		};

		let storage_value = T::decode_storage_value(&mut storage_value.as_slice())?;
		return Ok(Some(storage_value));
	}
}

#[derive(Clone)]
pub struct StorageMapIterator<T: StorageMap> {
	client: Client,
	phantom: PhantomData<T>,
	block_hash: H256,
	fetched_keys: Vec<String>,
	last_key: Option<String>,
	is_done: bool,
	prefix: String,
}

impl<T: StorageMap> StorageMapIterator<T> {
	pub fn new(client: Client, block_hash: H256) -> Self {
		Self {
			client,
			phantom: PhantomData::<T>,
			block_hash,
			fetched_keys: Vec::new(),
			last_key: None,
			is_done: false,
			prefix: hex::encode(T::encode_partial_key()),
		}
	}

	pub async fn next(&mut self) -> Result<Option<T::VALUE>, avail_rust_core::Error> {
		if self.is_done {
			return Ok(None);
		}

		// Fetch new keys
		if self.fetched_keys.is_empty() {
			self.fetched_keys = self
				.client
				.rpc_api()
				.state_get_keys_paged(
					Some(self.prefix.clone()),
					3,
					self.last_key.clone(),
					Some(self.block_hash),
				)
				.await?;

			if self.fetched_keys.is_empty() {
				self.is_done = true;
				return Ok(None);
			}

			self.fetched_keys.reverse();
		}

		let Some(storage_key) = self.fetched_keys.last() else {
			return Ok(None);
		};

		let storage_value = self
			.client
			.rpc_api()
			.state_get_storage(storage_key, Some(self.block_hash))
			.await?;

		let Some(storage_value) = storage_value else {
			return Ok(None);
		};

		let storage_value = T::decode_storage_value(&mut storage_value.as_slice())?;

		self.last_key = Some(storage_key.clone());
		self.fetched_keys.pop();

		Ok(Some(storage_value))
	}
}
