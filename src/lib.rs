#![forbid(unsafe_code)]

//! Settlement core for the Prism distributed transaction ledger.
//!
//! The crate provides signed escrow openings, beneficiary-authorized releases,
//! settlement routing, and deterministic state commitments for a single-asset DTL.

pub mod amount;
mod codec;
pub mod crypto;
pub mod error;
pub mod escrow;
pub mod ids;
pub mod ledger;
pub mod rebate;

pub use amount::Amount;
pub use crypto::{KeyPair, PublicIdentity, PublicKeyBytes, SignatureBytes};
pub use error::{PrismError, PrismResult};
pub use escrow::{
    EscrowRelease, EscrowTerms, ReleaseAuthorization, SettlementPath, SettlementQuote,
    SignedEscrowOpen, SignedEscrowRelease,
};
pub use ids::{AccountId, AssetId, Digest, EscrowId, TxId};
pub use ledger::{AccountState, EscrowState, PrismLedger, SupplySnapshot};
pub use rebate::{BondedRebatePolicy, BondedRebateQuote, quote_bonded_rebate};
