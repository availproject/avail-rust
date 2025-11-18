use crate::platform::sleep;
use std::{fmt::Debug, time::Duration};

#[cfg(feature = "tracing")]
pub(crate) fn trace_warn(message: &str) {
	tracing::warn!(target: "lib", message);
}

/// Repeatedly executes an asynchronous operation until it succeeds or retries are exhausted.
///
/// # Arguments
///
/// * `f` - Factory producing a future that performs the operation.
/// * `retry_on_error` - When `true`, the function sleeps and retries on failure.
///
/// # Returns
///
/// Returns the successful output of `f` or propagates the last encountered error.
///
/// # Errors
///
/// Returns the final error emitted by `f` once no retries remain.
///
/// # Examples
///
/// ```no_run
/// use avail_rust_client::utils::with_retry_on_error;
///
/// async fn fetch_value() -> Result<u32, &'static str> {
///     Err("transient failure")
/// }
///
/// async fn run() -> Result<u32, &'static str> {
///     with_retry_on_error(fetch_value, true).await
/// }
/// ```
pub async fn with_retry_on_error<F, Fut, O, E>(f: F, retry_on_error: bool) -> Result<O, E>
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
				trace_warn(&std::format!("Retrying after error: {:?}; next attempt in {}s", err, duration));
				sleep(Duration::from_secs(duration)).await;
			},
		};
	}
}

/// Executes an asynchronous operation, retrying on errors and optionally on `None` results.
///
/// # Arguments
///
/// * `f` - Factory producing a future that returns `Option<O>`.
/// * `retry_on_error` - Controls whether errors trigger retries.
/// * `retry_on_none` - When `true`, `None` results trigger retries until exhausted.
///
/// # Returns
///
/// Returns `Ok(Some(O))` on success, `Ok(None)` if no value was produced, or the last error emitted.
///
/// # Errors
///
/// Propagates the final error returned by `f` after exhausting retries.
///
/// # Examples
///
/// ```no_run
/// use avail_rust_client::utils::with_retry_on_error_and_none;
///
/// async fn maybe_fetch() -> Result<Option<u32>, &'static str> {
///     Ok(None)
/// }
///
/// async fn run() -> Result<Option<u32>, &'static str> {
///     with_retry_on_error_and_none(maybe_fetch, true, true).await
/// }
/// ```
pub async fn with_retry_on_error_and_none<F, Fut, O, E>(
	f: F,
	retry_on_error: bool,
	retry_on_none: bool,
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
				trace_warn(&std::format!(
					"Received None result; retrying in {}s because retry_on_none is enabled",
					duration
				));
				sleep(Duration::from_secs(duration)).await;
			},
			Err(err) if !retry_on_error => {
				return Err(err);
			},
			Err(err) => {
				let Some(duration) = sleep_duration.pop() else {
					return Err(err);
				};

				#[cfg(feature = "tracing")]
				trace_warn(&std::format!(
					"Retrying after error while awaiting Option result: {:?}; next attempt in {}s",
					err,
					duration
				));
				sleep(Duration::from_secs(duration)).await;
			},
		};
	}
}
