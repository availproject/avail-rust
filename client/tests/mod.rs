mod pallets;
mod subs;

#[tokio::test]
async fn run_tests() {
	//pallets::run_tests().await.unwrap();
	subs::run_tests().await.unwrap();
}
