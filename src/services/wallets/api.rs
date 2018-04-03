use exonum::crypto::{Hash, PublicKey};
use iron::prelude::*;
use exonum::api::{Api};
use router::Router;
use exonum::node::{ApiSender};
use exonum::blockchain::{Blockchain};
use exonum::encoding::serialize::FromHex;
use iron::status::Status;
use iron::headers::ContentType;
use iron::modifiers::Header;
use std::net;
use failure::Fail;

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
                let e: net::AddrParseError = net::AddrParseError {0: ()};
                return Err(
                    IronError::new(ApiError::InvalidParse, (
                        Status::BadRequest,
                        Header(ContentType::json()),
                        "\"Invalid request param: `pub_key`\"",
                    ))
                )
            }
        };
        let public_key = PublicKey::from_hex(hex_wallet_key).map_err(|e| {
            IronError::new(e, (
                Status::BadRequest,
                Header(ContentType::json()),
                "\"Invalid request param: `pub_key`\"",
            ))
        })?;

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

/// Error codes for Wallets service
#[derive(Debug, Fail)]
#[repr(u8)]
enum ApiError {
    /// Insufficient currency amount.
    ///
    /// Can be emitted by `TxTransfer`.
    #[fail(display = "Insufficient currency amount")]
    InvalidParse = 0,
}

impl iron::error::Error for ApiError {
    fn description(&self) -> &str {
        (self as Fail).cause()
    }
}
