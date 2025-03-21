use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::{
	body::{Buf, Bytes, Incoming},
	client::conn::http1::SendRequest,
	Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum Error {
	RequestError(RequestError),
	Serde(serde_json::Error),
	Custom(String),
}

impl From<RequestError> for Error {
	fn from(value: RequestError) -> Self {
		Self::RequestError(value)
	}
}

impl From<serde_json::Error> for Error {
	fn from(value: serde_json::Error) -> Self {
		Self::Serde(value)
	}
}

pub struct TurboDA {
	con: Connection,
	api_key: String,
}

impl TurboDA {
	pub async fn new(endpoint: &str, api_key: String) -> Result<Self, ConnectionError> {
		let con = Connection::new(endpoint).await?;
		Ok(Self { con, api_key })
	}

	pub fn url(&self) -> hyper::Uri {
		self.con.url()
	}

	pub async fn submit_raw_data(&mut self, data: Vec<u8>) -> Result<v1::SubmitDataRes, Error> {
		let d = Bytes::from(data);
		let config = RequestConfig {
			path: "/v1/submit_raw_data",
			method: Method::POST,
			headers: vec![("x-api-key", &self.api_key)],
		};

		let res = self.con.request(config, d).await?;
		let res = serde_json::from_str(&res)?;

		Ok(res)
	}

	pub async fn get_submission_info(&mut self, submission_id: String) -> Result<v1::SubmissionInfo, Error> {
		let path = format!("/v1/get_submission_info?submission_id={}", submission_id);
		let config = RequestConfig {
			path: &path,
			method: Method::GET,
			headers: vec![("x-api-key", &self.api_key)],
		};

		let result = self.con.request(config, Bytes::new()).await?;
		let json: v1::SubmissionInfoJson = serde_json::from_str(&result)?;

		if let Some(e) = json.error {
			return Err(Error::Custom(e));
		}

		if json.state == "Pending" {
			let value = v1::PendingSubmissionInfo::from(json);
			return Ok(v1::SubmissionInfo::Pending(value));
		} else if json.state == "Included" {
			let value = v1::IncludedSubmissionInfo::try_from(json).map_err(|e| Error::Custom(e))?;
			return Ok(v1::SubmissionInfo::Included(value));
		} else if json.state == "Finalized" {
			let value = v1::FinalizedSubmissionInfo::try_from(json).map_err(|e| Error::Custom(e))?;
			return Ok(v1::SubmissionInfo::Finalized(value));
		}

		unimplemented!()
	}
}

#[derive(Debug)]
pub enum ConnectionError {
	InvalidUri(hyper::http::uri::InvalidUri),
	NoHost,
	Io(std::io::Error),
	Handshake(hyper::Error),
	NoAuthority,
}

impl From<hyper::http::uri::InvalidUri> for ConnectionError {
	fn from(value: hyper::http::uri::InvalidUri) -> Self {
		Self::InvalidUri(value)
	}
}

impl From<std::io::Error> for ConnectionError {
	fn from(value: std::io::Error) -> Self {
		Self::Io(value)
	}
}

#[derive(Debug)]
pub enum RequestError {
	Builder(hyper::http::Error),
	Send(hyper::Error),
	ReadingBody(hyper::Error),
	Io(std::io::Error),
	ResponseNotOk(StatusCode),
}

pub struct Connection {
	sender: SendRequest<Full<Bytes>>,
	url: hyper::Uri,
	authority: hyper::http::uri::Authority,
}

impl Connection {
	pub async fn new(endpoint: &str) -> Result<Self, ConnectionError> {
		let url = format!("{}", endpoint).parse::<hyper::Uri>()?;

		let host = url.host().ok_or(ConnectionError::NoHost)?;
		let port = url.port_u16().unwrap_or(80);
		let address = format!("{}:{}", host, port);

		let stream = TcpStream::connect(address).await?;
		let io = TokioIo::new(stream);

		let handshake = hyper::client::conn::http1::handshake::<_, Full<Bytes>>(io).await;
		let handshake = handshake.map_err(|e| ConnectionError::Handshake(e))?;
		let (sender, conn) = handshake;

		tokio::task::spawn(async move {
			if let Err(err) = conn.await {
				log::error!("Connection failed: {:?}", err);
			}
		});

		let authority = url.authority().ok_or(ConnectionError::NoAuthority)?.clone();

		Ok(Self { sender, url, authority })
	}

	pub fn url(&self) -> hyper::Uri {
		return self.url.clone();
	}

	pub async fn request_raw<'a>(
		&mut self,
		config: RequestConfig<'a>,
		data: Bytes,
	) -> Result<Response<Incoming>, RequestError> {
		let mut req = Request::builder().uri(config.path).method(config.method);
		for (key, value) in config.headers {
			req = req.header(key, value);
		}

		let req = req
			.header(hyper::header::HOST, self.authority.as_str())
			.body(Full::<Bytes>::new(data));
		let req = req.map_err(|e| RequestError::Builder(e))?;

		let res = self.sender.send_request(req).await;
		let res = res.map_err(|e| RequestError::Send(e))?;

		Ok(res)
	}

	pub async fn request<'a>(&mut self, config: RequestConfig<'a>, data: Bytes) -> Result<String, RequestError> {
		use std::io::Read;
		let raw_res = self.request_raw(config, data).await?;

		let status = raw_res.status();
		println!("Response: {}", status);
		println!("Headers: {:#?}\n", raw_res.headers());

		if status != hyper::http::status::StatusCode::OK {
			return Err(RequestError::ResponseNotOk(status));
		}

		let body = raw_res.into_body();
		let buffer = body.collect().await.map_err(|e| RequestError::ReadingBody(e))?;
		let buffer = buffer.aggregate();

		let mut res = String::new();
		buffer
			.reader()
			.read_to_string(&mut res)
			.map_err(|e| RequestError::Io(e))?;

		Ok(res)
	}
}

#[derive(Clone)]
pub struct RequestConfig<'a> {
	pub path: &'a str,
	pub method: Method,
	pub headers: Vec<(&'a str, &'a str)>,
}

pub mod v1 {
	use super::*;

	#[derive(Debug, Clone, Deserialize)]
	pub struct SubmitDataRes {
		pub submission_id: String,
	}

	#[derive(Debug, Clone)]
	pub enum SubmissionInfo {
		Pending(PendingSubmissionInfo),
		Included(IncludedSubmissionInfo),
		Finalized(FinalizedSubmissionInfo),
		Error(String),
	}

	#[derive(Debug, Clone)]
	pub struct PendingSubmissionInfo {
		pub id: Arc<str>,
		pub user_id: Arc<str>,
		pub created_at: Arc<str>,
		pub amount_data: Arc<str>,
	}

	impl From<SubmissionInfoJson> for PendingSubmissionInfo {
		fn from(value: SubmissionInfoJson) -> Self {
			PendingSubmissionInfo {
				id: value.id.into(),
				user_id: value.data.user_id.into(),
				created_at: value.data.created_at.into(),
				amount_data: value.data.amount_data.into(),
			}
		}
	}

	#[derive(Debug, Clone)]
	pub struct IncludedSubmissionInfo {
		pub id: Arc<str>,
		pub user_id: Arc<str>,
		pub created_at: Arc<str>,
		pub amount_data: Arc<str>,
		pub block_hash: Arc<str>,
		pub block_height: u32,
		pub tx_hash: Arc<str>,
		pub tx_index: u32,
		pub data_billed: Arc<str>,
		pub data_hash: Arc<str>,
		pub fees: Arc<str>,
	}

	impl TryFrom<SubmissionInfoJson> for IncludedSubmissionInfo {
		type Error = String;

		fn try_from(value: SubmissionInfoJson) -> Result<Self, Self::Error> {
			let Some(block_hash) = value.data.block_hash else {
				return Err(String::from("Field block_hash is not set."));
			};

			let Some(block_height) = value.data.block_number else {
				return Err(String::from("Field block_number is not set."));
			};

			let Some(tx_hash) = value.data.tx_hash else {
				return Err(String::from("Field tx_hash is not set."));
			};

			let Some(tx_index) = value.data.tx_index else {
				return Err(String::from("Field tx_index is not set."));
			};

			let Some(data_billed) = value.data.data_billed else {
				return Err(String::from("Field data_billed is not set."));
			};

			let Some(data_hash) = value.data.data_hash else {
				return Err(String::from("Field data_hash is not set."));
			};

			let Some(fees) = value.data.fees else {
				return Err(String::from("Field fees is not set."));
			};

			let res = IncludedSubmissionInfo {
				id: value.id.into(),
				user_id: value.data.user_id.into(),
				created_at: value.data.created_at.into(),
				amount_data: value.data.amount_data.into(),
				block_hash: block_hash.into(),
				block_height,
				tx_hash: tx_hash.into(),
				tx_index,
				data_billed: data_billed.into(),
				data_hash: data_hash.into(),
				fees: fees.into(),
			};

			Ok(res)
		}
	}

	#[derive(Debug, Clone)]
	pub struct FinalizedSubmissionInfo {
		pub id: Arc<str>,
		pub user_id: Arc<str>,
		pub created_at: Arc<str>,
		pub amount_data: Arc<str>,
		pub block_hash: Arc<str>,
		pub block_height: u32,
		pub tx_hash: Arc<str>,
		pub tx_index: u32,
		pub data_billed: Arc<str>,
		pub data_hash: Arc<str>,
		pub fees: Arc<str>,
	}

	impl TryFrom<SubmissionInfoJson> for FinalizedSubmissionInfo {
		type Error = String;

		fn try_from(value: SubmissionInfoJson) -> Result<Self, Self::Error> {
			let Some(block_hash) = value.data.block_hash else {
				return Err(String::from("Field block_hash is not set."));
			};

			let Some(block_height) = value.data.block_number else {
				return Err(String::from("Field block_number is not set."));
			};

			let Some(tx_hash) = value.data.tx_hash else {
				return Err(String::from("Field tx_hash is not set."));
			};

			let Some(tx_index) = value.data.tx_index else {
				return Err(String::from("Field tx_index is not set."));
			};

			let Some(data_billed) = value.data.data_billed else {
				return Err(String::from("Field data_billed is not set."));
			};

			let Some(data_hash) = value.data.data_hash else {
				return Err(String::from("Field data_hash is not set."));
			};

			let Some(fees) = value.data.fees else {
				return Err(String::from("Field fees is not set."));
			};

			let res = FinalizedSubmissionInfo {
				id: value.id.into(),
				user_id: value.data.user_id.into(),
				created_at: value.data.created_at.into(),
				amount_data: value.data.amount_data.into(),
				block_hash: block_hash.into(),
				block_height,
				tx_hash: tx_hash.into(),
				tx_index,
				data_billed: data_billed.into(),
				data_hash: data_hash.into(),
				fees: fees.into(),
			};

			Ok(res)
		}
	}

	#[derive(Debug, Clone, Deserialize)]
	pub struct SubmissionInfoJson {
		pub data: SubmissionInfoDataJson,
		pub error: Option<String>,
		pub id: String,
		pub state: String,
	}

	#[derive(Debug, Clone, Deserialize)]
	pub struct SubmissionInfoDataJson {
		pub amount_data: String,
		pub block_hash: Option<String>,
		pub block_number: Option<u32>,
		pub created_at: String,
		pub data_billed: Option<String>,
		pub data_hash: Option<String>,
		pub fees: Option<String>,
		pub tx_hash: Option<String>,
		pub tx_index: Option<u32>,
		pub user_id: String,
	}
}
