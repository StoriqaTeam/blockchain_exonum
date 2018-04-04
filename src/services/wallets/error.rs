use exonum::blockchain::ExecutionError;

/// Error codes for Wallets service
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Insufficient currency amount.
    ///
    /// Can be emitted by `TxTransfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 0,

    /// Triggered when sender's account nonce isn't equal to transaction nonce.
    ///
    /// Can be emitted by `TxTransfer`.
    #[fail(display = "Invalid nonce")]
    InvalidNonce = 1,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        ExecutionError::new(value as u8)
    }
}
