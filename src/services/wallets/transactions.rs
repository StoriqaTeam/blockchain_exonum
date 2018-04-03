use exonum::blockchain::{Transaction, ExecutionResult};
use exonum::storage::{Fork};
use exonum::messages::{Message};

use exonum::crypto::{PublicKey};
use super::super::{WALLETS_SERVICE_ID};

encoding_struct! {
    /// Wallet struct used to persist data within the service.
    struct Wallet {
        /// Current balance.
        balance: u64,
    }
}

/// Additional methods for managing balance of the wallet in an immutable fashion.
impl Wallet {
    /// Returns a copy of this wallet with the balance increased by the specified amount.
    pub fn increase(self, amount: u64) -> Self {
        let balance = self.balance() + amount;
        Self::new(balance)
    }

    /// Returns a copy of this wallet with the balance decreased by the specified amount.
    pub fn decrease(self, amount: u64) -> Self {
        assert!(self.balance() >= amount);
        let balance = self.balance() - amount;
        Self::new(balance)
    }
}

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

    /// Retrieves two wallets to apply the transfer; they should be previously registered
    /// with the help of [`TxCreateWallet`] transactions. Checks the sender's
    /// balance and applies changes to the balances of the wallets if the sender's balance
    /// is sufficient. Otherwise, performs no op.
    ///
    /// [`TxCreateWallet`]: struct.TxCreateWallet.html
    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        unimplemented!()
    }
}
