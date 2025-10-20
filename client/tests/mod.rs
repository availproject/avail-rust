mod pallets;

#[tokio::test]
async fn pallet_tests() {
	pallets::run_tests().await.unwrap();
}
