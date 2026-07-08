use thiserror::Error;

use crate::{AccountId, Amount, AssetId, Digest, EscrowId, TxId};

pub type PrismResult<T> = Result<T, PrismError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PrismError {
    #[error("amount overflow")]
    AmountOverflow,

    #[error("amount underflow")]
    AmountUnderflow,

    #[error("amount {value} exceeds protocol max {max}")]
    AmountOutOfRange { value: u128, max: u128 },

    #[error("zero amount")]
    ZeroAmount,

    #[error("invalid public key")]
    InvalidPublicKey,

    #[error("invalid signature")]
    InvalidSignature,

    #[error("serialization failed: {0}")]
    Serialization(String),

    #[error("identity mismatch for account {0}")]
    IdentityMismatch(AccountId),

    #[error("account not found: {0}")]
    AccountNotFound(AccountId),

    #[error("account already exists: {0}")]
    AccountAlreadyExists(AccountId),

    #[error("escrow not found: {0}")]
    EscrowNotFound(EscrowId),

    #[error("escrow already exists: {0}")]
    EscrowAlreadyExists(EscrowId),

    #[error("escrow is already released: {0}")]
    EscrowReleased(EscrowId),

    #[error("unauthorized escrow depositor: expected {expected}, received {received}")]
    UnauthorizedDepositor {
        expected: AccountId,
        received: AccountId,
    },

    #[error("unauthorized release signer: expected {expected}, received {received}")]
    UnauthorizedRelease {
        expected: AccountId,
        received: AccountId,
    },

    #[error("nonce mismatch for {account}: expected {expected}, received {received}")]
    NonceMismatch {
        account: AccountId,
        expected: u64,
        received: u64,
    },

    #[error("nonce overflow")]
    NonceOverflow,

    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(TxId),

    #[error("insufficient funds for {account}: available {available}, required {required}")]
    InsufficientFunds {
        account: AccountId,
        available: Amount,
        required: Amount,
    },

    #[error("asset mismatch: expected {expected}, received {received}")]
    AssetMismatch {
        expected: AssetId,
        received: AssetId,
    },

    #[error("terms hash mismatch for escrow {escrow_id}: expected {expected}, received {received}")]
    TermsHashMismatch {
        escrow_id: EscrowId,
        expected: Digest,
        received: Digest,
    },

    #[error("policy violation: {0}")]
    Policy(String),

    #[error("conservation failure for {asset}: expected {expected}, observed {observed}")]
    Conservation {
        asset: AssetId,
        expected: Amount,
        observed: Amount,
    },
}
