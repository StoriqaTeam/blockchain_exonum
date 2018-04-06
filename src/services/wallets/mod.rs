//! Defines service for wallets operations.

pub mod api;
pub mod repo;
pub mod error;

use exonum::encoding;
use exonum::messages::{Message, RawTransaction};
use exonum::storage::{Fork, Snapshot};
use exonum::blockchain::{ApiContext, ExecutionResult, Service, Transaction, TransactionSet};
use exonum::crypto::{gen_keypair, Hash};
use exonum::encoding::serialize::ToHex;
use exonum::api::Api;
use iron::Handler;
use router::Router;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::json::reexport::Value;

use self::api::WalletsApi;
use self::repo::WalletsRepo;
use self::error::Error;
use self::repo::Wallet;
use super::{WALLETS_SERVICE_ID, WALLETS_SERVICE_NAME};

pub struct WalletsService;

impl Service for WalletsService {
    fn service_name(&self) -> &'static str {
        WALLETS_SERVICE_NAME
    }

    fn service_id(&self) -> u16 {
        WALLETS_SERVICE_ID
    }

    fn initialize(&self, fork: &mut Fork) -> Value {
        let (public_key, private_key) = gen_keypair();
        let string_public_key = public_key.to_string();
        let mut string_private_key = String::new();
        private_key.write_hex(&mut string_private_key).unwrap();
        println!(
            "Keys with money - public: {}, private: {}",
            string_public_key, string_private_key
        );
        let mut repo = WalletsRepo::new(fork);
        repo.as_mut().put(&public_key, Wallet::new(0, 1000000000));
        Value::Null
    }

    // Implement a method to deserialize transactions coming to the node.
    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let tx = WalletsTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    // Hashes for the service tables that will be included into the state hash.
    fn state_hash(&self, view: &Snapshot) -> Vec<Hash> {
        vec![WalletsRepo::new(view).as_read_only().root_hash()]
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

// Need to keep it here, because WalletsTransactions in transactions! macro is private
transactions! {
    WalletsTransactions {
        const SERVICE_ID = WALLETS_SERVICE_ID;

        /// Transaction type for transferring tokens between two wallets. If the receiving
        /// wallet doesn't exist - it will be created
        ///
        /// See [the `Transaction` trait implementation](#impl-Transaction) for details how
        /// `TxTransfer` transactions are processed.
        struct TxTransfer {
            /// Public key of the sender.
            from: &PublicKey,
            /// Public key of the receiver.
            to: &PublicKey,
            /// Number of tokens to transfer from sender's account to receiver's account.
            amount: u64,
            /// Number of transaction from sender's wallet. Must be equal to sender's account nonce.
            nonce: u64,
        }
    }
}

impl Transaction for TxTransfer {
    /// Checks if the sender is not the receiver, and checks correctness of the
    /// sender's signature.
    fn verify(&self) -> bool {
        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    /// Retrieves two wallets to apply the transfer; if receiving wallet doesn't exist
    /// it is created. Checks the sender's
    /// balance and applies changes to the balances of the wallets if the sender's balance
    /// is sufficient. Otherwise, performs no op.
    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut repo = WalletsRepo::new(view);
        let mut repo_mut = repo.as_mut();
        let sender_wallet = match repo_mut.get(self.from()) {
            Some(val) => val,
            None => Err(Error::InsufficientCurrencyAmount)?,
        };
        if sender_wallet.nonce() != self.nonce() {
            return Err(Error::InvalidNonce.into());
        }
        let receiver_wallet = repo_mut.get(self.to()).unwrap_or_default();
        let amount = self.amount();
        if sender_wallet.balance() >= amount {
            let sender_wallet = sender_wallet.decrease(amount);
            let receiver_wallet = receiver_wallet.increase(amount);
            repo_mut.put(self.from(), sender_wallet);
            repo_mut.put(self.to(), receiver_wallet);
            Ok(())
        } else {
            Err(Error::InsufficientCurrencyAmount)?
        }
    }
}
