use crate::subxt_rpcs::RpcConfig;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction for a avail node.
pub type AvailExtrinsicParams<T> = avail_rust_core::DefaultExtrinsicParams<T>;

#[derive(Clone, Debug, Default)]
pub struct AvailConfig;

impl crate::subxt_core::Config for AvailConfig {
	type AccountId = avail_rust_core::AccountId;
	type Address = avail_rust_core::MultiAddress;
	type AssetId = u32;
	type ExtrinsicParams = AvailExtrinsicParams<Self>;
	type Hash = avail_rust_core::BlockHash;
	type Hasher = avail_rust_core::BlakeTwo256;
	type Header = avail_rust_core::AvailHeader;
	type Signature = avail_rust_core::MultiSignature;
}

impl RpcConfig for AvailConfig {
	type AccountId = avail_rust_core::AccountId;
	type Hash = avail_rust_core::BlockHash;
	type Header = avail_rust_core::AvailHeader;
}
