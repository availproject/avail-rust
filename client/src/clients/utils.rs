use std::{fmt::Debug, sync::Arc, time::Duration};

use crate::AsyncOp;

#[cfg(feature = "tracing")]
pub(crate) fn trace_warn(message: &str) {
	tracing::warn!(target: "lib", message);
}

pub async fn with_retry_on_error<F, Fut, O, E>(f: F, retry_on_error: bool, op: Arc<dyn AsyncOp>) -> Result<O, E>
where
	F: Fn() -> Fut,
	Fut: Future<Output = Result<O, E>>,
	E: Debug,
{
	let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
	if !retry_on_error {
		sleep_duration.clear();
	}

	loop {
		match f().await {
			Ok(x) => return Ok(x),
			Err(err) => {
				let Some(duration) = sleep_duration.pop() else {
					return Err(err);
				};

				#[cfg(feature = "tracing")]
				trace_warn(&std::format!("{:?}", err));
				op.sleep(Duration::from_secs(duration)).await;
			},
		};
	}
}

pub async fn with_retry_on_error_and_none<F, Fut, O, E>(
	f: F,
	retry_on_error: bool,
	retry_on_none: bool,
	op: Arc<dyn AsyncOp>,
) -> Result<Option<O>, E>
where
	F: Fn() -> Fut,
	Fut: Future<Output = Result<Option<O>, E>>,
	E: Debug,
{
	let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
	loop {
		match f().await {
			Ok(Some(x)) => return Ok(Some(x)),
			Ok(None) if !retry_on_none => {
				return Ok(None);
			},
			Ok(None) => {
				let Some(duration) = sleep_duration.pop() else {
					return Ok(None);
				};

				#[cfg(feature = "tracing")]
				trace_warn(&std::format!("Error: {}", "Something TODO"));
				op.sleep(Duration::from_secs(duration)).await;
			},
			Err(err) if !retry_on_error => {
				return Err(err);
			},
			Err(err) => {
				let Some(duration) = sleep_duration.pop() else {
					return Err(err);
				};

				#[cfg(feature = "tracing")]
				trace_warn(&std::format!("{:?}", err));
				op.sleep(Duration::from_secs(duration)).await;
			},
		};
	}
}
