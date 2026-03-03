#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorOperation {
	ConversionToHash,
	ConversionToAccountId,
	ChainLegacyBlockEvents,
	ChainBlockState,
	ChainBlockInfoFrom,
	ChainBlockAuthor,
	ChainBlockEventCount,
	ChainBlockWeight,
	ChainBlockJustification,
	ChainFetchExtrinsics,
	ChainFetchEvents,
	ChainBlockTimestamp,
	BlockEventsExtrinsicWeight,
	BlockExtrinsicTyped,
	BlockExtrinsicFromRpc,
	BlockSharedHashNumber,
	BlockSharedHeader,
	SubmissionWaitForReceipt,
	BlockJustification,
	SecretUriParse,
	KeypairParse,
}

impl ErrorOperation {
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::ConversionToHash => "CONVERSION_TO_HASH",
			Self::ConversionToAccountId => "CONVERSION_TO_ACCOUNT_ID",
			Self::ChainLegacyBlockEvents => "CHAIN_LEGACY_BLOCK_EVENTS",
			Self::ChainBlockState => "CHAIN_BLOCK_STATE",
			Self::ChainBlockInfoFrom => "CHAIN_BLOCK_INFO_FROM",
			Self::ChainBlockAuthor => "CHAIN_BLOCK_AUTHOR",
			Self::ChainBlockEventCount => "CHAIN_BLOCK_EVENT_COUNT",
			Self::ChainBlockWeight => "CHAIN_BLOCK_WEIGHT",
			Self::ChainBlockJustification => "CHAIN_BLOCK_JUSTIFICATION",
			Self::ChainFetchExtrinsics => "CHAIN_FETCH_EXTRINSICS",
			Self::ChainFetchEvents => "CHAIN_FETCH_EVENTS",
			Self::ChainBlockTimestamp => "CHAIN_BLOCK_TIMESTAMP",
			Self::BlockEventsExtrinsicWeight => "BLOCK_EVENTS_EXTRINSIC_WEIGHT",
			Self::BlockExtrinsicTyped => "BLOCK_EXTRINSIC_TYPED",
			Self::BlockExtrinsicFromRpc => "BLOCK_EXTRINSIC_FROM_RPC",
			Self::BlockSharedHashNumber => "BLOCK_SHARED_HASH_NUMBER",
			Self::BlockSharedHeader => "BLOCK_SHARED_HEADER",
			Self::SubmissionWaitForReceipt => "SUBMISSION_WAIT_FOR_RECEIPT",
			Self::BlockJustification => "BLOCK_JUSTIFICATION",
			Self::SecretUriParse => "SECRET_URI_PARSE",
			Self::KeypairParse => "KEYPAIR_PARSE",
		}
	}

	pub fn parse(value: &str) -> Option<Self> {
		match value {
			"CONVERSION_TO_HASH" => Some(Self::ConversionToHash),
			"CONVERSION_TO_ACCOUNT_ID" => Some(Self::ConversionToAccountId),
			"CHAIN_LEGACY_BLOCK_EVENTS" => Some(Self::ChainLegacyBlockEvents),
			"CHAIN_BLOCK_STATE" => Some(Self::ChainBlockState),
			"CHAIN_BLOCK_INFO_FROM" => Some(Self::ChainBlockInfoFrom),
			"CHAIN_BLOCK_AUTHOR" => Some(Self::ChainBlockAuthor),
			"CHAIN_BLOCK_EVENT_COUNT" => Some(Self::ChainBlockEventCount),
			"CHAIN_BLOCK_WEIGHT" => Some(Self::ChainBlockWeight),
			"CHAIN_BLOCK_JUSTIFICATION" => Some(Self::ChainBlockJustification),
			"CHAIN_FETCH_EXTRINSICS" => Some(Self::ChainFetchExtrinsics),
			"CHAIN_FETCH_EVENTS" => Some(Self::ChainFetchEvents),
			"CHAIN_BLOCK_TIMESTAMP" => Some(Self::ChainBlockTimestamp),
			"BLOCK_EVENTS_EXTRINSIC_WEIGHT" => Some(Self::BlockEventsExtrinsicWeight),
			"BLOCK_EXTRINSIC_TYPED" => Some(Self::BlockExtrinsicTyped),
			"BLOCK_EXTRINSIC_FROM_RPC" => Some(Self::BlockExtrinsicFromRpc),
			"BLOCK_SHARED_HASH_NUMBER" => Some(Self::BlockSharedHashNumber),
			"BLOCK_SHARED_HEADER" => Some(Self::BlockSharedHeader),
			"SUBMISSION_WAIT_FOR_RECEIPT" => Some(Self::SubmissionWaitForReceipt),
			"BLOCK_JUSTIFICATION" => Some(Self::BlockJustification),
			"SECRET_URI_PARSE" => Some(Self::SecretUriParse),
			"KEYPAIR_PARSE" => Some(Self::KeypairParse),
			_ => None,
		}
	}
}

impl core::fmt::Display for ErrorOperation {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.write_str(self.as_str())
	}
}
