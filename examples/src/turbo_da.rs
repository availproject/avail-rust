/* use avail_rust::prelude::*;
use std::time::Duration;

pub async fn run() -> Result<(), ClientError> {
	let endpoint = std::env::var("ENDPOINT").unwrap();
	let api_key = std::env::var("KEY").unwrap();

	let client = TurboDA::new(&endpoint, api_key).await;
	let mut client = client.unwrap();

	// URL
	println!("URL: {:?}", client.url());

	// Submitting
	let res = client.submit_raw_data("Lala".into()).await.unwrap();
	dbg!(&res);

	tokio::time::sleep(Duration::from_secs(30)).await;

	// Fetching submission info
	let res = client.get_submission_info(res.submission_id).await.unwrap();
	dbg!(&res);

	Ok(())
}
 */
