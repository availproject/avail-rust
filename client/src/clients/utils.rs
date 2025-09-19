use crate::platform::sleep;
use std::time::Duration;

#[cfg(feature = "tracing")]
pub(crate) fn trace_warn(message: &str) {
	tracing::warn!(target: "lib", message);
}

#[cfg(feature = "tracing")]
pub(crate) async fn sleep_on_retry(duration: u64, message: &str, value: &str) {
	trace_warn(&std::format!("Message: {}, Value: {}, Sleep for {} seconds", message, value, duration));
	sleep(Duration::from_secs(duration)).await;
}

#[cfg(not(feature = "tracing"))]
pub(crate) async fn sleep_on_retry(duration: u64, _message: &str, _value: &str) {
	sleep(Duration::from_secs(duration)).await;
}

pub async fn with_retry_on_error<F, Fut, O, E>(f: F, retry_on_error: bool, error_message: &str) -> Result<O, E>
where
	F: Fn() -> Fut,
	Fut: Future<Output = Result<O, E>>,
	E: ToString,
{
	let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
	loop {
		match f().await {
			Ok(x) => return Ok(x),
			Err(err) if !retry_on_error => {
				return Err(err);
			},
			Err(err) => {
				let Some(duration) = sleep_duration.pop() else {
					return Err(err);
				};
				sleep_on_retry(duration, error_message, &err.to_string()).await;
			},
		};
	}
}

pub async fn with_retry_on_error_and_none<F, Fut, O, E>(
	f: F,
	retry_on_error: bool,
	retry_on_none: bool,
	error_message: &str,
) -> Result<Option<O>, E>
where
	F: Fn() -> Fut,
	Fut: Future<Output = Result<Option<O>, E>>,
	E: ToString,
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
				sleep_on_retry(duration, error_message, "Option<None>").await;
			},
			Err(err) if !retry_on_error => {
				return Err(err);
			},
			Err(err) => {
				let Some(duration) = sleep_duration.pop() else {
					return Err(err);
				};
				sleep_on_retry(duration, error_message, &err.to_string()).await;
			},
		};
	}
}
