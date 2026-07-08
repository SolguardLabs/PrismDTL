#![forbid(unsafe_code)]

//! Núcleo de liquidación del libro de transacciones distribuido Prism.
//!
//! El crate proporciona aperturas de escrow firmadas, liberaciones autorizadas
//! por beneficiario, enrutamiento de liquidación y compromisos de estado
//! deterministas para un entorno DTL de activo único.

pub mod amount;
mod codec;
pub mod crypto;
pub mod error;
pub mod escrow;
pub mod ids;
pub mod ledger;

pub use amount::Amount;
pub use crypto::{KeyPair, PublicIdentity, PublicKeyBytes, SignatureBytes};
pub use error::{PrismError, PrismResult};
pub use escrow::{
    EscrowRelease, EscrowTerms, ReleaseAuthorization, SettlementPath, SignedEscrowOpen,
    SignedEscrowRelease,
};
pub use ids::{AccountId, AssetId, Digest, EscrowId, TxId};
pub use ledger::{AccountState, EscrowState, PrismLedger};
