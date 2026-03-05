use crate::{
	Client, Error, RetryPolicy, UserError,
	block::{
		BlockExtrinsicMetadata,
		events::{BlockEvents, BlockEventsQuery},
		shared::BlockContext,
	},
	error_ops,
};
use avail_rust_core::{
	Extrinsic, ExtrinsicDecodable, H256, HasHeader, HashNumber, MultiAddress, RpcError,
	rpc::{self, AllowedExtrinsic, DataFormat},
	substrate::extrinsic::Preamble,
	types::HashStringNumber,
};
use codec::Decode;

/// Unified query for fetching extrinsics from a block.
///
/// Provides both untyped methods (`get`, `first`, `last`, `all`, `count`, `exists`)
/// that return [`BlockEncodedExtrinsic`] with raw call bytes, and typed `_as` variants
/// (`get_as`, `first_as`, `last_as`, `all_as`) that decode the call into a concrete
/// Rust struct.
pub struct BlockExtrinsicsQuery {
	ctx: BlockContext,
}

impl BlockExtrinsicsQuery {
	pub fn new(client: Client, at: HashStringNumber) -> Self {
		Self { ctx: BlockContext::new(client, at) }
	}

	// ── Untyped (encoded) methods ───────────────────────────────────────

	pub async fn get(&self, extrinsic_id: impl Into<HashStringNumber>) -> Result<Option<UntypedExtrinsic>, Error> {
		async fn inner(
			s: &BlockExtrinsicsQuery,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<UntypedExtrinsic>, Error> {
			let allowed = match extrinsic_id {
				HashStringNumber::Hash(x) => AllowedExtrinsic::from(x),
				HashStringNumber::String(x) => AllowedExtrinsic::try_from(x.as_str()).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => AllowedExtrinsic::from(x),
			};

			s.first(Some(vec![allowed]), Default::default()).await
		}

		inner(self, extrinsic_id.into()).await
	}

	pub async fn first(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
	) -> Result<Option<UntypedExtrinsic>, Error> {
		let at = self.ctx.hash_number()?;
		let chain = self.ctx.chain();

		let mut result = chain
			.extrinsics(at, allow_list, sig_filter, DataFormat::Extrinsic)
			.await?;

		let Some(info) = result.first_mut() else {
			return Ok(None);
		};

		let ext = UntypedExtrinsic::from_rpc_extrinsic(info, at)?;
		Ok(Some(ext))
	}

	pub async fn last(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
	) -> Result<Option<UntypedExtrinsic>, Error> {
		let at = self.ctx.hash_number()?;
		let chain = self.ctx.chain();

		let mut result = chain
			.extrinsics(at, allow_list, sig_filter, DataFormat::Extrinsic)
			.await?;
		let Some(info) = result.last_mut() else {
			return Ok(None);
		};

		let ext = UntypedExtrinsic::from_rpc_extrinsic(info, at)?;
		Ok(Some(ext))
	}

	pub async fn all(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
	) -> Result<Vec<UntypedExtrinsic>, Error> {
		let at = self.ctx.hash_number()?;
		let chain = self.ctx.chain();

		let extrinsics = chain
			.extrinsics(at, allow_list, sig_filter, DataFormat::Extrinsic)
			.await?;

		let mut result = Vec::with_capacity(extrinsics.len());
		for info in extrinsics {
			let ext = UntypedExtrinsic::from_rpc_extrinsic(&info, at)?;
			result.push(ext);
		}

		Ok(result)
	}

	pub async fn count(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
	) -> Result<usize, Error> {
		let at = self.ctx.at.clone();
		let chain = self.ctx.chain();
		let result = chain.extrinsics(at, allow_list, sig_filter, DataFormat::None).await?;

		Ok(result.len())
	}

	pub async fn exists(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
	) -> Result<bool, Error> {
		self.count(allow_list, sig_filter).await.map(|x| x > 0)
	}

	// ── Typed (_as) methods ─────────────────────────────────────────────

	pub async fn get_as<T: HasHeader + Decode>(
		&self,
		extrinsic_id: impl Into<HashStringNumber>,
	) -> Result<Option<TypedExtrinsic<T>>, Error> {
		async fn inner<T: HasHeader + Decode>(
			s: &BlockExtrinsicsQuery,
			extrinsic_id: HashStringNumber,
		) -> Result<Option<TypedExtrinsic<T>>, Error> {
			let allowed = match extrinsic_id {
				HashStringNumber::Hash(x) => AllowedExtrinsic::from(x),
				HashStringNumber::String(x) => AllowedExtrinsic::try_from(x.as_str()).map_err(UserError::Decoding)?,
				HashStringNumber::Number(x) => AllowedExtrinsic::from(x),
			};

			let encoded = s.first(Some(vec![allowed]), Default::default()).await?;
			let Some(encoded) = encoded else {
				return Ok(None);
			};

			Ok(Some(encoded.as_typed::<T>()?))
		}

		inner::<T>(self, extrinsic_id.into()).await
	}

	pub async fn first_as<T: HasHeader + Decode>(
		&self,
		sig_filter: rpc::SignatureFilter,
	) -> Result<Option<TypedExtrinsic<T>>, Error> {
		let allow_list = Some(vec![T::HEADER_INDEX.into()]);

		let encoded = self.first(allow_list, sig_filter).await?;
		let Some(encoded) = encoded else {
			return Ok(None);
		};

		Ok(Some(encoded.as_typed::<T>()?))
	}

	pub async fn last_as<T: HasHeader + Decode>(
		&self,
		sig_filter: rpc::SignatureFilter,
	) -> Result<Option<TypedExtrinsic<T>>, Error> {
		let allow_list = Some(vec![T::HEADER_INDEX.into()]);

		let encoded = self.last(allow_list, sig_filter).await?;
		let Some(encoded) = encoded else {
			return Ok(None);
		};

		Ok(Some(encoded.as_typed::<T>()?))
	}

	pub async fn all_as<T: HasHeader + Decode>(
		&self,
		sig_filter: rpc::SignatureFilter,
	) -> Result<Vec<TypedExtrinsic<T>>, Error> {
		let allow_list = Some(vec![T::HEADER_INDEX.into()]);

		let all = self.all(allow_list, sig_filter).await?;
		let mut result = Vec::with_capacity(all.len());
		for encoded in all {
			result.push(encoded.as_typed::<T>()?);
		}

		Ok(result)
	}

	// ── Raw RPC access ──────────────────────────────────────────────────

	pub async fn rpc(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
		data_format: rpc::DataFormat,
	) -> Result<Vec<rpc::Extrinsic>, Error> {
		self.ctx
			.chain()
			.extrinsics(self.ctx.at.clone(), allow_list, sig_filter, data_format)
			.await
	}

	// ── Configuration ───────────────────────────────────────────────────

	pub fn set_retry_policy(&mut self, value: RetryPolicy) {
		self.ctx.set_retry_policy(value);
	}

	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}
}

// ── BlockEncodedExtrinsic ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UntypedExtrinsic {
	pub preamble: Preamble,
	pub call: Vec<u8>,
	pub metadata: BlockExtrinsicMetadata,
}

impl UntypedExtrinsic {
	pub fn new(preamble: Preamble, call: Vec<u8>, metadata: BlockExtrinsicMetadata) -> Self {
		Self { preamble, call, metadata }
	}

	pub async fn events(&self, client: Client) -> Result<BlockEvents, Error> {
		let events = BlockEventsQuery::new(client, self.metadata.at)
			.extrinsic(self.ext_index())
			.await?;

		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};

		Ok(events)
	}

	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	pub fn nonce(&self) -> Option<u32> {
		match &self.preamble {
			Preamble::Bare(_) => None,
			Preamble::Signed(_, _, extension) => Some(extension.nonce),
			Preamble::General(_, extension) => Some(extension.nonce),
		}
	}

	pub fn tip(&self) -> Option<u128> {
		match &self.preamble {
			Preamble::Bare(_) => None,
			Preamble::Signed(_, _, extension) => Some(extension.tip),
			Preamble::General(_, extension) => Some(extension.tip),
		}
	}

	pub fn ss58_address(&self) -> Option<String> {
		match &self.preamble {
			Preamble::Bare(_) => None,
			Preamble::Signed(address, _, _) => match address {
				MultiAddress::Id(a) => Some(std::format!("{}", a)),
				_ => None,
			},
			Preamble::General(_, _) => None,
		}
	}

	pub fn as_typed<T: HasHeader + Decode>(self) -> Result<TypedExtrinsic<T>, Error> {
		TypedExtrinsic::<T>::try_from(self)
			.map_err(|e| Error::decode_with_op(error_ops::ErrorOperation::BlockExtrinsicTyped, e))
	}

	pub fn is<T: HasHeader>(&self) -> bool {
		self.metadata.pallet_id == T::HEADER_INDEX.0 && self.metadata.variant_id == T::HEADER_INDEX.1
	}

	pub fn header(&self) -> (u8, u8) {
		(self.metadata.pallet_id, self.metadata.variant_id)
	}

	pub fn from_rpc_extrinsic(ext: &rpc::Extrinsic, at: HashNumber) -> Result<Self, Error> {
		let metadata = BlockExtrinsicMetadata::from_rpc_extrinsic(ext, at);
		let extrinsic = Extrinsic::try_from(ext.data.as_str())
			.map_err(|e| Error::decode_with_op(error_ops::ErrorOperation::BlockExtrinsicFromRpc, e))?;
		Ok(UntypedExtrinsic::new(extrinsic.preamble, extrinsic.call.0, metadata))
	}
}

// ── BlockExtrinsic<T> ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TypedExtrinsic<T: HasHeader + Decode> {
	pub preamble: Preamble,
	pub call: T,
	pub metadata: BlockExtrinsicMetadata,
}

impl<T: HasHeader + Decode> TypedExtrinsic<T> {
	pub fn new(preamble: Preamble, call: T, metadata: BlockExtrinsicMetadata) -> Self {
		Self { preamble, call, metadata }
	}

	pub async fn events(&self, client: Client) -> Result<BlockEvents, Error> {
		let events = BlockEventsQuery::new(client, self.metadata.at)
			.extrinsic(self.ext_index())
			.await?;

		if events.is_empty() {
			return Err(RpcError::ExpectedData("No events found for extrinsic".into()).into());
		};

		Ok(events)
	}

	pub fn ext_index(&self) -> u32 {
		self.metadata.ext_index
	}

	pub fn ext_hash(&self) -> H256 {
		self.metadata.ext_hash
	}

	pub fn nonce(&self) -> Option<u32> {
		match &self.preamble {
			Preamble::Bare(_) => None,
			Preamble::Signed(_, _, extension) => Some(extension.nonce),
			Preamble::General(_, extension) => Some(extension.nonce),
		}
	}

	pub fn tip(&self) -> Option<u128> {
		match &self.preamble {
			Preamble::Bare(_) => None,
			Preamble::Signed(_, _, extension) => Some(extension.tip),
			Preamble::General(_, extension) => Some(extension.tip),
		}
	}

	pub fn ss58_address(&self) -> Option<String> {
		match &self.preamble {
			Preamble::Bare(_) => None,
			Preamble::Signed(address, _, _) => match address {
				MultiAddress::Id(a) => Some(std::format!("{}", a)),
				_ => None,
			},
			Preamble::General(_, _) => None,
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<UntypedExtrinsic> for TypedExtrinsic<T> {
	type Error = String;

	fn try_from(value: UntypedExtrinsic) -> Result<Self, Self::Error> {
		let call = T::from_call(value.call)?;
		Ok(Self::new(value.preamble, call, value.metadata))
	}
}

impl<T: HasHeader + Decode> TryFrom<&UntypedExtrinsic> for TypedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &UntypedExtrinsic) -> Result<Self, Self::Error> {
		let call = T::from_call(&value.call)?;
		Ok(Self::new(value.preamble.clone(), call, value.metadata.clone()))
	}
}
