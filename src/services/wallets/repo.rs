//! Models and repo (data manipulation service) for Wallets service

use exonum::storage::{Fork, ProofMapIndex, Snapshot};
use exonum::crypto::PublicKey;

// servicename.tablename
const TABLE_NAME: &str = "wallets.wallets";

encoding_struct! {
    /// Wallet struct used to persist data within the service.
    struct Wallet {
        /// Total number of transactions from this account
        nonce: u64,
        /// Current balance.
        balance: u64,
    }
}

/// Additional methods for managing balance of the wallet in an immutable fashion.
impl Wallet {
    /// Returns a copy of this wallet with the balance increased by the specified amount.
    pub fn increase(self, amount: u64) -> Self {
        let balance = self.balance() + amount;
        Self::new(self.nonce() + 1, balance)
    }

    /// Returns a copy of this wallet with the balance decreased by the specified amount.
    pub fn decrease(self, amount: u64) -> Self {
        debug_assert!(self.balance() >= amount);
        let balance = self.balance() - amount;
        Self::new(self.nonce() + 1, balance)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        // TODO: remove this hack
        Wallet::new(0, 0)
    }
}

/// Repo for manipulating wallets data.
pub struct WalletsRepo<T> {
    view: T,
}

/// Readonly methods for WallerRepo
impl<T: AsRef<Snapshot>> WalletsRepo<T> {
    /// Creates a new schema instance.
    pub fn new(view: T) -> Self {
        WalletsRepo { view }
    }

    /// Gets a read-only view on the table
    pub fn as_read_only(&self) -> ProofMapIndex<&Snapshot, PublicKey, Wallet> {
        ProofMapIndex::new(TABLE_NAME, self.view.as_ref())
    }
}

/// Mutating methods for WalletRepo
impl<'a> WalletsRepo<&'a mut Fork> {
    /// Get mutable methods for wallets
    pub fn as_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Wallet> {
        ProofMapIndex::new(TABLE_NAME, &mut self.view)
    }
}
