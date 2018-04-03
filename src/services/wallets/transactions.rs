use exonum::blockchain::{Transaction, ExecutionResult};
use exonum::storage::{Fork};
use exonum::messages::{Message};

use exonum::crypto::{PublicKey};
use super::super::{WALLETS_SERVICE_ID};
use super::repo::WalletsRepo;
use super::error::Error;

transactions! {
    CurrencyTransactions {
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
            /// Auxiliary number to guarantee non-idempotence of transactions.
            seed: u64,
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
        let sender_wallet = match repo.get(self.from()) {
            Some(val) => val,
            None => Err(Error::InsufficientCurrencyAmount)?,
        };
        let receiver_wallet = repo.get(self.to()).unwrap_or_default();
        let amount = self.amount();
        if sender_wallet.balance() >= amount {
            let sender_wallet = sender_wallet.decrease(amount);
            let receiver_wallet = receiver_wallet.increase(amount);
            let mut repo_mut = repo.as_mut();
            repo_mut.put(self.from(), sender_wallet);
            repo_mut.put(self.to(), receiver_wallet);
            Ok(())
        } else {
            Err(Error::InsufficientCurrencyAmount)?
        }

    }
}
