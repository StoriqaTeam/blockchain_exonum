pub mod transactions;
pub mod api;

use exonum::encoding;
use exonum::messages::RawTransaction;
use exonum::storage::{Snapshot};
use exonum::blockchain::{Service, Transaction, ApiContext};
use exonum::crypto::{Hash};
use exonum::api::{Api};
use iron::Handler;
use router::Router;
use self::api::WalletsApi;

pub struct WalletsService;

use super::{WALLETS_SERVICE_ID, WALLETS_SERVICE_NAME};

impl Service for WalletsService {
    fn service_name(&self) -> &'static str {
        WALLETS_SERVICE_NAME
    }

    fn service_id(&self) -> u16 {
        WALLETS_SERVICE_ID
    }

    // Implement a method to deserialize transactions coming to the node.
    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
      unimplemented!()
        // let tx = CurrencyTransactions::tx_from_raw(raw)?;
        // Ok(tx.into())
    }

    // Hashes for the service tables that will be included into the state hash.
    // To simplify things, we don't have [Merkelized tables][merkle] in the service storage
    // for now, so we return an empty vector.
    //
    // [merkle]: https://exonum.com/doc/architecture/storage/#merklized-indices
    fn state_hash(&self, _: &Snapshot) -> Vec<Hash> {
        vec![]
    }

    // Create a REST `Handler` to process web requests to the node.
    fn public_api_handler(&self, ctx: &ApiContext) -> Option<Box<Handler>> {
        let mut router = Router::new();
        let api = WalletsApi {
            channel: ctx.node_channel().clone(),
            blockchain: ctx.blockchain().clone(),
        };
        api.wire(&mut router);
        Some(Box::new(router))
    }
}
