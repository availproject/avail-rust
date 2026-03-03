/// Controls retry behavior for SDK operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RetryPolicy {
	/// Use the parent/default retry setting.
	#[default]
	Inherit,
	/// Always retry.
	Enabled,
	/// Never retry.
	Disabled,
}

impl RetryPolicy {
	pub(crate) fn resolve(self, inherited: bool) -> bool {
		match self {
			Self::Enabled => true,
			Self::Disabled => false,
			Self::Inherit => inherited,
		}
	}
}
