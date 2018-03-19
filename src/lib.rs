extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::{Blockchain, Service, Transaction, ApiContext};
use exonum::encoding::serialize::FromHex;
use exonum::node::{TransactionSend, ApiSender};
use exonum::messages::{RawTransaction, Message};
use exonum::storage::{Fork, MapIndex, Snapshot};
use exonum::crypto::{Hash, PublicKey};
use exonum::encoding;
use exonum::api::{Api, ApiError};
use iron::prelude::*;
use iron::Handler;
use router::Router;
use serde::Deserialize;

// Service identifier
const SERVICE_ID: u16 = 1;
// Identifier for wallet creation transaction type
const TX_CREATE_WALLET_ID: u16 = 1;
// Identifier for coins transfer transaction type
const TX_TRANSFER_ID: u16 = 2;
// Starting balance of a newly created wallet
const INIT_BALANCE: u64 = 100;

encoding_struct! {
    struct Wallet {
        pub_key: &PublicKey,
        name: &str,
        balance: u64,
    }
}

impl Wallet {
    pub fn increase(self, amount: u64) -> Self {
        let balance = self.balance() + amount;
        Self::new(self.pub_key(), self.name(), balance)
    }

    pub fn decrease(self, amount: u64) -> Self {
        let balance = self.balance() - amount;
        Self::new(self.pub_key(), self.name(), balance)
    }
}

pub struct CurrencySchema<T> {
    view: T,
}

impl<T: AsRef<Snapshot>> CurrencySchema<T> {
    pub fn new(view: T) -> Self {
        CurrencySchema { view }
    }

    pub fn wallets(&self) -> MapIndex<&Snapshot, PublicKey, Wallet> {
        MapIndex::new("cryptocurrency.wallets", self.view.as_ref())
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }
}

impl<'a> CurrencySchema<&'a mut Fork> {
    pub fn wallets_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        MapIndex::new("cryptocurrency.wallets", &mut self.view)
    }
}

message! {
    struct TxCreateWallet {
        const TYPE = SERVICE_ID;
        const ID = TX_CREATE_WALLET_ID;

        pub_key: &PublicKey,
        name: &str,
    }
}

impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema::new(view);
        if schema.wallet(self.pub_key()).is_none() {
            let wallet = Wallet::new(
                self.pub_key(),
                self.name(),
                INIT_BALANCE);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets_mut().put(self.pub_key(), wallet);
        }
    }
}

message! {
    struct TxTransfer {
        const TYPE = SERVICE_ID;
        const ID = TX_TRANSFER_ID;

        from: &PublicKey,
        to: &PublicKey,
        amount: u64,
        seed: u64,
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
         (*self.from() != *self.to()) &&
             self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        let sender = schema.wallet(self.from());
        let receiver = schema.wallet(self.to());
        if let (Some(sender), Some(receiver)) = (sender, receiver) {
            let amount = self.amount();
            if sender.balance() >= amount {
                let sender = sender.decrease(amount);
                let receiver = receiver.increase(amount);
                println!("Transfer between wallets: {:?} => {:?}",
                    sender,
                    receiver);
                let mut wallets = schema.wallets_mut();
                wallets.put(self.from(), sender);
                wallets.put(self.to(), receiver);
            }
        }
    }
}

#[derive(Clone)]
struct CryptocurrencyApi {
    channel: ApiSender,
    blockchain: Blockchain,
}

impl CryptocurrencyApi {
    fn post_transaction<T>(&self, req: &mut Request) -> IronResult<Response>
    where
        T: Transaction + Clone + for<'de> Deserialize<'de>,
    {
        match req.get::<bodyparser::Struct<T>>() {
            Ok(Some(transaction)) => {
                let transaction: Box<Transaction> = Box::new(transaction);
                let tx_hash = transaction.hash();
                self.channel.send(transaction).map_err(ApiError::from)?;
                self.ok_response(&json!({
                    "tx_hash": tx_hash
                }))
            }
            Ok(None) => Err(ApiError::IncorrectRequest(
                "Empty request body".into(),
            ))?,
            Err(e) => Err(ApiError::IncorrectRequest(Box::new(e)))?,
        }
    }

    fn get_wallet(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path();
        let wallet_key = path.last().unwrap();
        let public_key = PublicKey::from_hex(wallet_key)
            .map_err(ApiError::FromHex)?;

        let wallet = {
            let snapshot = self.blockchain.snapshot();
            let schema = CurrencySchema::new(snapshot);
            schema.wallet(&public_key)
        };

        if let Some(wallet) = wallet {
            self.ok_response(&serde_json::to_value(wallet).unwrap())
        } else {
            self.not_found_response(
                &serde_json::to_value("Wallet not found").unwrap(),
            )
        }
    }

    fn get_wallets(&self, _: &mut Request) -> IronResult<Response> {
        let snapshot = self.blockchain.snapshot();
        let schema = CurrencySchema::new(snapshot);
        let idx = schema.wallets();
        let wallets: Vec<Wallet> = idx.values().collect();
        self.ok_response(&serde_json::to_value(&wallets).unwrap())
    }
}

impl Api for CryptocurrencyApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let post_create_wallet = move |req: &mut Request| {
            self_.post_transaction::<TxCreateWallet>(req)
        };
        let self_ = self.clone();
        let post_transfer = move |req: &mut Request| {
            self_.post_transaction::<TxTransfer>(req)
        };
        let self_ = self.clone();
        let get_wallets = move |req: &mut Request| self_.get_wallets(req);
        let self_ = self.clone();
        let get_wallet = move |req: &mut Request| self_.get_wallet(req);

        // Bind handlers to specific routes.
        router.post("/v1/wallets", post_create_wallet, "post_create_wallet");
        router.post("/v1/wallets/transfer", post_transfer, "post_transfer");
        router.get("/v1/wallets", get_wallets, "get_wallets");
        router.get("/v1/wallet/:pub_key", get_wallet, "get_wallet");
    }
}

pub struct CurrencyService;

impl Service for CurrencyService {
    fn service_name(&self) -> &'static str { "cryptocurrency" }

    fn service_id(&self) -> u16 { SERVICE_ID }

    fn tx_from_raw(&self, raw: RawTransaction)
        -> Result<Box<Transaction>, encoding::Error> {

        let trans: Box<Transaction> = match raw.message_type() {
            TX_TRANSFER_ID => Box::new(TxTransfer::from_raw(raw)?),
            TX_CREATE_WALLET_ID => Box::new(TxCreateWallet::from_raw(raw)?),
            _ => {
                return Err(encoding::Error::IncorrectMessageType {
                    message_type: raw.message_type()
                });
            },
        };
        Ok(trans)
    }

    fn state_hash(&self, _: &Snapshot) -> Vec<Hash> {
        vec![]
    }

    fn public_api_handler(&self, ctx: &ApiContext) -> Option<Box<Handler>> {
        let mut router = Router::new();
        let api = CryptocurrencyApi {
            channel: ctx.node_channel().clone(),
            blockchain: ctx.blockchain().clone(),
        };
        api.wire(&mut router);
        Some(Box::new(router))
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
