use exonum::crypto::{Hash};
use iron::prelude::*;
use exonum::api::{Api};
use router::Router;
use exonum::node::{ApiSender};
use exonum::blockchain::{Blockchain};

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
    fn get_wallet(&self, req: &mut Request) -> IronResult<Response> {
        unimplemented!()
    }

    /// Endpoint for dumping all wallets from the storage.
    fn get_wallets(&self, _: &mut Request) -> IronResult<Response> {
        unimplemented!()
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
