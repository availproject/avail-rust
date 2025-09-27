use avail_rust::{self, Keypair, KeypairExt, Options, SubmittedTransaction};
use std::{collections::HashMap, ffi::CStr};
use tokio::runtime::Runtime;

struct GlobalClient {
	pub avail_client: avail_rust::Client,
	pub runtime: Runtime,
	pub signer_handles: HashMap<i32, avail_rust::Keypair>,
	pub submitted_tx_handles: HashMap<i32, SubmittedTransaction>,
	pub receipt_handles: HashMap<i32, avail_rust::TransactionReceipt>,
}

impl GlobalClient {
	pub fn new(avail_client: avail_rust::Client, runtime: Runtime) -> Self {
		Self {
			avail_client,
			runtime,
			signer_handles: Default::default(),
			submitted_tx_handles: Default::default(),
			receipt_handles: Default::default(),
		}
	}
}

static GLOBAL_CLIENT: std::sync::Mutex<Option<GlobalClient>> = std::sync::Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn hello_from_rust() {
	println!("Hello from Rust!");
}

#[unsafe(no_mangle)]
pub extern "C" fn initialize_client(endpoint: *const std::ffi::c_char) {
	println!("Rust: initialize_client");
	let endpoint = unsafe { CStr::from_ptr(endpoint) };
	let endpoint = endpoint.to_string_lossy().to_string();

	let runtime = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();
	let avail_client = runtime.block_on(async { avail_rust::Client::new(&endpoint).await.unwrap() });
	{
		let mut lock = GLOBAL_CLIENT.lock().unwrap();
		*lock = Some(GlobalClient::new(avail_client, runtime))
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn initialize_signer(secret_seed: *const std::ffi::c_char) -> std::ffi::c_int {
	println!("Rust: initialize_signer");
	let secret_seed = unsafe { CStr::from_ptr(secret_seed) };
	let secret_seed = secret_seed.to_string_lossy().to_string();

	let signer = Keypair::from_str(&secret_seed).unwrap();
	let id = {
		let mut lock = GLOBAL_CLIENT.lock().unwrap();
		let global = lock.as_mut().unwrap();

		let id: i32 = global.signer_handles.len() as i32;
		global.signer_handles.insert(id, signer);
		id
	};

	id
}

#[unsafe(no_mangle)]
pub extern "C" fn do_submit_data(signer_handle: i32, data: *const std::ffi::c_char, app_id: i32) -> std::ffi::c_int {
	println!("Rust: do_submit_data");
	let data = unsafe { CStr::from_ptr(data) };
	let data = data.to_string_lossy().to_string();

	let id = {
		let mut lock = GLOBAL_CLIENT.lock().unwrap();
		let global = lock.as_mut().unwrap();

		let signer = global.signer_handles.get(&signer_handle).unwrap();
		let submitted_tx = global.runtime.block_on(async {
			let tx = global.avail_client.tx().data_availability().submit_data(data);
			tx.sign_and_submit(signer, Options::new(app_id as u32)).await.unwrap()
		});
		let id = global.submitted_tx_handles.len() as i32;
		global.submitted_tx_handles.insert(id, submitted_tx);
		id
	};

	id
}

#[unsafe(no_mangle)]
pub extern "C" fn get_transaction_receipt(submitted_tx_handle: i32) -> i32 {
	println!("Rust: get_transaction_receipt");
	let id = {
		let mut lock = GLOBAL_CLIENT.lock().unwrap();
		let global = lock.as_mut().unwrap();

		let submitted_tx = global.submitted_tx_handles.get(&submitted_tx_handle).unwrap();
		let receipt = global
			.runtime
			.block_on(async { submitted_tx.receipt(true).await.unwrap().unwrap() });
		let id = global.receipt_handles.len() as i32;
		global.receipt_handles.insert(id, receipt);
		id
	};

	id
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Receipt {
	pub block_height: i32,
	pub block_hash: [u8; 32],
	pub transaction_index: i32,
	pub transaction_hash: [u8; 32],
	pub handle: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn receipt_new(receipt_handle: i32) -> *mut Receipt {
	println!("Rust: receipt_new");
	let id = {
		let mut lock = GLOBAL_CLIENT.lock().unwrap();
		let global = lock.as_mut().unwrap();

		let receipt = global.receipt_handles.get(&receipt_handle).cloned().unwrap();
		let r = Receipt {
			block_height: receipt.block_ref.height as i32,
			block_hash: receipt.block_ref.hash.0,
			transaction_index: receipt.tx_ref.index as i32,
			transaction_hash: receipt.tx_ref.hash.0,
			handle: receipt_handle,
		};
		Box::into_raw(Box::new(r))
	};

	id
}

#[unsafe(no_mangle)]
pub extern "C" fn receipt_free(ptr: *mut Receipt) {
	println!("Rust: receipt_free");
	{
		let mut lock = GLOBAL_CLIENT.lock().unwrap();
		let global = lock.as_mut().unwrap();

		let b = unsafe { Box::from_raw(ptr) };
		global.receipt_handles.remove(&b.handle);
	};
}

#[unsafe(no_mangle)]
pub extern "C" fn receipt_block_height(ptr: *const Receipt) -> i32 {
	println!("Rust: receipt_block_height");
	unsafe { ptr.read().block_height }
}

#[unsafe(no_mangle)]
pub extern "C" fn receipt_block_hash(ptr: *const Receipt) -> *const std::ffi::c_uchar {
	println!("Rust: receipt_block_hash");
	let r = unsafe { &*ptr };
	r.block_hash.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn receipt_transaction_index(ptr: *const Receipt) -> i32 {
	println!("Rust: receipt_transaction_index");
	unsafe { ptr.read().transaction_index }
}

#[unsafe(no_mangle)]
pub extern "C" fn receipt_transaction_hash(ptr: *const Receipt) -> *const std::ffi::c_uchar {
	println!("Rust: receipt_transaction_hash");
	let r = unsafe { &*ptr };
	r.transaction_hash.as_ptr()
}
