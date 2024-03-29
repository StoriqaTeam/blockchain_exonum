use exonum::crypto::{Hash, PublicKey};
use iron::prelude::*;
use exonum::api::{Api, ApiError as ExonumApiError};
use router::Router;
use exonum::node::{ApiSender, TransactionSend};
use exonum::blockchain::{Blockchain, Transaction};
use exonum::encoding::serialize::FromHex;
use iron::status::Status;
use iron::headers::ContentType;
use iron::modifiers::Header;
use std;
use iron::error::Error as IronErrorTrait;
use std::fmt::{Display, Formatter};
use bodyparser;
use serde_json;
use super::WalletsTransactions;

use super::repo::WalletsRepo;

#[derive(Clone)]
pub struct WalletsApi {
    pub channel: ApiSender,
    pub blockchain: Blockchain,
}

/// The structure returned by the REST API.
#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Hash of the transaction.
    pub tx_hash: Hash,
}

impl WalletsApi {
    /// Endpoint for getting a single wallet.
    fn get_balance(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let hex_wallet_key = match path.last() {
            Some(key) => key,
            None => return Err(ApiError::InvalidParam("public_key", req.url.to_string()).into()),
        };
        let public_key = match PublicKey::from_hex(hex_wallet_key) {
            Ok(key) => key,
            Err(e) => {
                return Err(IronError::new(
                    e,
                    (
                        Status::BadRequest,
                        Header(ContentType::json()),
                        r#"{"error": "Invalid request param: `public_key`"}"#,
                    ),
                ))
            }
        };

        let wallet = {
            let snapshot = self.blockchain.snapshot();
            let repo = WalletsRepo::new(snapshot);
            let repo = repo.as_read_only();
            repo.get(&public_key).unwrap_or_default()
        };

        self.ok_response(&json!({ "balance": wallet.balance() }))
    }

    /// Common processing for transaction-accepting endpoints.
    fn post_transaction(&self, req: &mut Request) -> IronResult<Response> {
        match req.get::<bodyparser::Struct<WalletsTransactions>>() {
            Ok(Some(transaction)) => {
                let transaction: Box<Transaction> = transaction.into();
                let tx_hash = transaction.hash();
                self.channel
                    .send(transaction)
                    .map_err(ExonumApiError::from)?;
                let json = TransactionResponse { tx_hash };
                self.ok_response(&serde_json::to_value(&json).unwrap())
            }
            Ok(None) => Err(ApiError::UnprocessableEntity("Empty request body".to_string()).into()),
            Err(e) => Err(ApiError::Unknown(e.to_string()).into()),
        }
    }
}

/// `Api` trait implementation.
///
/// `Api` facilitates conversion between transactions/read requests and REST
/// endpoints; for example, it parses `POST`ed JSON into the binary transaction
/// representation used in Exonum internally.
impl Api for WalletsApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let post_transfer = move |req: &mut Request| self_.post_transaction(req);
        let self_ = self.clone();
        let get_wallet = move |req: &mut Request| self_.get_balance(req);

        // Bind handlers to specific routes.
        router.post("/v1/wallets/transfer", post_transfer, "post_transfer");
        router.get("/v1/wallet/:pub_key", get_wallet, "get_wallet");
    }
}

/// Error codes for Http server
#[derive(Debug)]
enum ApiError {
    /// Invalid prarmeter either in path or in query
    InvalidParam(&'static str, String),
    UnprocessableEntity(String),
    Unknown(String),
}

impl IronErrorTrait for ApiError {
    fn description(&self) -> &str {
        match self {
            &ApiError::InvalidParam(_, _) => "Invalid param in path or query",
            &ApiError::UnprocessableEntity(_) => "Unprocessable entity",
            &ApiError::Unknown(_) => "Unknown error",
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &ApiError::InvalidParam(key, ref value) => f.write_str(&format!(
                "{}: Path: {}, Query: {}",
                self.description(),
                key,
                value
            )),
            &ApiError::UnprocessableEntity(ref msg) => {
                f.write_str(&format!("{}: {}", self.description(), msg))
            }
            &ApiError::Unknown(ref msg) => f.write_str(&format!("{}: {}", self.description(), msg)),
        }
    }
}

impl Into<IronError> for ApiError {
    fn into(self) -> IronError {
        let message = { format!(r#"{{"error": "{}"}}"#, &self) };
        IronError::new(
            self,
            (Status::BadRequest, Header(ContentType::json()), message),
        )
    }
}
