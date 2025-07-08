use crate::{
	clients::Client,
	subxt_core::{self, storage::address::Address, utils::Yes},
};
use avail_rust_core::H256;

#[cfg(feature = "subxt")]
use crate::subxt::Error;
#[cfg(feature = "subxt")]
use crate::subxt::backend::StreamOfResults;
#[cfg(feature = "subxt")]
use crate::subxt::storage::StorageKeyValuePair;
#[cfg(feature = "subxt")]
use std::future::Future;

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
