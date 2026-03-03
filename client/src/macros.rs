macro_rules! retry {
	($should:expr, $body:expr) => {{
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		if !$should {
			sleep_duration.clear();
		}
		loop {
			match $body {
				Ok(x) => break Ok(x),
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						break Err(err);
					};
					#[cfg(feature = "tracing")]
					$crate::utils::trace_warn(&std::format!(
						"Retrying after error: {:?}; next attempt in {}s",
						err,
						duration
					));
					$crate::platform::sleep(std::time::Duration::from_secs(duration)).await;
				},
			}
		}
	}};
}

macro_rules! retry_or_none {
	($retry_on_error:expr, $retry_on_none:expr, $body:expr) => {{
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			match $body {
				Ok(Some(x)) => break Ok(Some(x)),
				Ok(None) if !$retry_on_none => break Ok(None),
				Ok(None) => {
					let Some(duration) = sleep_duration.pop() else {
						break Ok(None);
					};
					#[cfg(feature = "tracing")]
					$crate::utils::trace_warn(&std::format!(
						"Received None result; retrying in {}s because retry_on_none is enabled",
						duration
					));
					$crate::platform::sleep(std::time::Duration::from_secs(duration)).await;
				},
				Err(err) if !$retry_on_error => break Err(err),
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						break Err(err);
					};
					#[cfg(feature = "tracing")]
					$crate::utils::trace_warn(&std::format!(
						"Retrying after error while awaiting Option result: {:?}; next attempt in {}s",
						err,
						duration
					));
					$crate::platform::sleep(std::time::Duration::from_secs(duration)).await;
				},
			}
		}
	}};
}
