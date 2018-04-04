use exonum::crypto::{Hash, PublicKey};
use iron::prelude::*;
use exonum::api::Api;
use router::Router;
use exonum::node::ApiSender;
use exonum::blockchain::Blockchain;
use exonum::encoding::serialize::FromHex;
use iron::status::Status;
use iron::headers::ContentType;
use iron::modifiers::Header;
use std;
use failure::Fail;
use iron;
use iron::error::Error as IronErrorTrait;
use std::fmt::{Display, Formatter};

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
            None => {
                return Err(IronError::new(
                    ApiError::InvalidParam("public_key", req.url.to_string()),
                    (
                        Status::BadRequest,
                        Header(ContentType::json()),
                        r#"{"error": "Invalid request param: `public_key`"}"#,
                    ),
                ))
            }
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
        unimplemented!()
    }
}

/// `Api` trait implementation.
///
/// `Api` facilitates conversion between transactions/read requests and REST
/// endpoints; for example, it parses `POST`ed JSON into the binary transaction
/// representation used in Exonum internally.
impl Api for WalletsApi {
    fn wire(&self, router: &mut Router) {
        unimplemented!()
    }
}

/// Error codes for Http server
#[derive(Debug)]
enum ApiError {
    /// Invalid prarmeter either in path or in query
    InvalidParam(&'static str, String),
}

impl IronErrorTrait for ApiError {
    fn description(&self) -> &str {
        match self {
            &ApiError::InvalidParam(_, _) => "Invalid param in path or query",
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
        }
    }
}
