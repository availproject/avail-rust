use crate::{
	Client, Error, UserError,
	block::{encoded::ExtrinsicsOpts, shared::BlockContext},
};
use avail_rust_core::{
	EncodeSelector, ExtrinsicDecodable, HasHeader, RpcError,
	rpc::{self, ExtrinsicFilter},
	types::HashStringNumber,
};
use codec::Decode;

/// Detached view that decodes extrinsic call payloads within a block.
pub struct ExtrinsicCalls {
	ctx: BlockContext,
}

impl ExtrinsicCalls {
	/// Builds a helper dedicated to decoded extrinsic calls.
	///
	/// # Parameters
	/// - `client`: RPC client used to query extrinsic data.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Call-focused helper scoped to the target block.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Self { ctx: BlockContext::new(client, block_id.into()) }
	}

	/// Fetches a specific extrinsic call by hash, index, or string identifier.
	///
	/// # Parameters
	/// - `extrinsic_id`: Identifier used to select the extrinsic.
	///
	/// # Returns
	/// - `Ok(Some(T))`: The extrinsic matched the identifier and decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the identifier.
	/// - `Err(Error)`: Identifier decoding or the RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn get<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<T>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &ExtrinsicCalls,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<T>, Error> {
			let filter = match extrinsic_id {
				HashStringNumber::Hash(x) => ExtrinsicFilter::from(x),
				HashStringNumber::String(x) => ExtrinsicFilter::try_from(x).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => ExtrinsicFilter::from(x),
			};
			s.first::<T>(ExtrinsicsOpts::new().filter(filter)).await
		}

		inner::<T>(self, extrinsic_id.into()).await
	}

	/// Fetches the first extrinsic call that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to return.
	///
	/// # Returns
	/// - `Ok(Some(T))`: First matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: The RPC request or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn first<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Option<T>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.first_mut() else {
			return Ok(None);
		};

		let Some(data) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let data = T::from_call(data).map_err(Error::Other)?;

		Ok(Some(data))
	}

	/// Fetches the last extrinsic call that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsic to return.
	///
	/// # Returns
	/// - `Ok(Some(T))`: Final matching extrinsic decoded as `T`.
	/// - `Ok(None)`: No extrinsic satisfied the filters.
	/// - `Err(Error)`: The RPC request or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn last<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Option<T>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let mut result = chain.system_fetch_extrinsics(block_id, opts).await?;

		let Some(first) = result.last_mut() else {
			return Ok(None);
		};

		let Some(data) = first.data.take() else {
			return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
		};

		let data = T::from_call(data).map_err(Error::Other)?;

		Ok(Some(data))
	}

	/// Fetches every extrinsic call that matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to collect.
	///
	/// # Returns
	/// - `Ok(Vec<T>)`: Zero or more decoded extrinsic calls.
	/// - `Err(Error)`: The RPC request or payload decoding failed.
	///
	/// # Side Effects
	/// - Performs an RPC call, decoding each payload, and may retry according to the retry policy.
	pub async fn all<T: HasHeader + Decode>(&self, mut opts: ExtrinsicsOpts) -> Result<Vec<T>, Error> {
		if opts.filter.is_none() {
			opts.filter = Some(T::HEADER_INDEX.into())
		}
		let opts = opts.to_rpc_opts(EncodeSelector::Call);

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let extrinsics = chain.system_fetch_extrinsics(block_id, opts).await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for ext in extrinsics {
			let Some(data) = ext.data else {
				return Err(Error::RpcError(RpcError::ExpectedData("Expected data for encoded extrinsic.".into())));
			};

			let data = T::from_call(data).map_err(Error::Other)?;

			result.push(data);
		}

		Ok(result)
	}

	/// Counts how many extrinsic calls match the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to count.
	///
	/// # Returns
	/// - `Ok(usize)`: Number of matching extrinsics.
	/// - `Err(Error)`: The RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call and may retry according to the retry policy.
	pub async fn count<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<usize, Error> {
		let mut opts: rpc::ExtrinsicOpts = opts.to_rpc_opts(EncodeSelector::None);
		opts.transaction_filter = T::HEADER_INDEX.into();

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let result = chain.system_fetch_extrinsics(block_id, opts).await?;

		Ok(result.len())
	}

	/// Reports whether any extrinsic matches the supplied filters.
	///
	/// # Parameters
	/// - `opts`: Filters describing which extrinsics to test.
	///
	/// # Returns
	/// - `Ok(true)`: At least one matching extrinsic exists.
	/// - `Ok(false)`: No extrinsics matched the filters.
	/// - `Err(Error)`: The RPC request failed.
	///
	/// # Side Effects
	/// - Performs an RPC call via [`ExtrinsicCalls::count`] and may retry according to the retry policy.
	pub async fn exists<T: HasHeader>(&self, opts: ExtrinsicsOpts) -> Result<bool, Error> {
		self.count::<T>(opts).await.map(|x| x > 0)
	}

	/// Overrides the retry behaviour for future extrinsic-call lookups.
	///
	/// # Parameters
	/// - `value`: `Some(true)` to force retries, `Some(false)` to disable retries, `None` to inherit the client default.
	///
	/// # Returns
	/// - `()`: The override is stored for subsequent operations.
	///
	/// # Side Effects
	/// - Updates the internal retry setting used by follow-up RPC calls.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.ctx.set_retry_on_error(value);
	}

	/// Reports whether call lookups retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}
}
