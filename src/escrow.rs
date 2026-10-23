use serde::{Deserialize, Serialize};

use crate::{
    AccountId, Amount, AssetId, Digest, EscrowId, KeyPair, PrismError, PrismResult, PublicIdentity,
    SignatureBytes, TxId, crypto::verify,
};

pub const OPEN_DOMAIN: &str = "escrow-open-v3";
pub const RELEASE_DOMAIN: &str = "escrow-release-v3";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettlementPath {
    pub operator: AccountId,
    pub operator_fee: Amount,
    pub rebate_recipient: Option<AccountId>,
    pub rebate_amount: Amount,
    pub routing_epoch: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettlementQuote {
    pub gross_amount: Amount,
    pub beneficiary_amount: Amount,
    pub operator_fee: Amount,
    pub rebate_amount: Amount,
    pub total_charges: Amount,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EscrowTerms {
    pub network_id: u32,
    pub escrow_id: EscrowId,
    pub depositor: AccountId,
    pub beneficiary: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub depositor_nonce: u64,
    pub expires_at_epoch: u64,
    pub memo_hash: Digest,
    pub settlement_path: SettlementPath,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
struct EscrowAuthorizationView {
    network_id: u32,
    escrow_id: EscrowId,
    depositor: AccountId,
    beneficiary: AccountId,
    asset: AssetId,
    amount: Amount,
    depositor_nonce: u64,
    expires_at_epoch: u64,
    memo_hash: Digest,
    settlement_path: SettlementPath,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EscrowRelease {
    pub network_id: u32,
    pub escrow_id: EscrowId,
    pub beneficiary: AccountId,
    pub release_nonce: u64,
    pub observed_terms_hash: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
struct ReleaseAuthorizationView {
    network_id: u32,
    escrow_id: EscrowId,
    beneficiary: AccountId,
    release_nonce: u64,
    observed_terms_hash: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedEscrowOpen {
    pub depositor: PublicIdentity,
    pub terms: EscrowTerms,
    pub signature: SignatureBytes,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedEscrowRelease {
    pub signer: PublicIdentity,
    pub release: EscrowRelease,
    pub signature: SignatureBytes,
}

pub struct ReleaseAuthorization;

impl SettlementPath {
    pub fn direct(operator: AccountId) -> Self {
        Self {
            operator,
            operator_fee: Amount::zero(),
            rebate_recipient: None,
            rebate_amount: Amount::zero(),
            routing_epoch: 0,
        }
    }

    pub fn quote(self, gross_amount: Amount) -> PrismResult<SettlementQuote> {
        let total_charges = self.operator_fee.checked_add(self.rebate_amount)?;
        if total_charges > gross_amount {
            return Err(PrismError::Policy(
                "settlement path exceeds escrow value".to_owned(),
            ));
        }

        Ok(SettlementQuote {
            gross_amount,
            beneficiary_amount: gross_amount.checked_sub(total_charges)?,
            operator_fee: self.operator_fee,
            rebate_amount: self.rebate_amount,
            total_charges,
        })
    }
}

impl EscrowTerms {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        depositor: AccountId,
        beneficiary: AccountId,
        asset: AssetId,
        amount: Amount,
        depositor_nonce: u64,
        expires_at_epoch: u64,
        memo_hash: Digest,
        settlement_path: SettlementPath,
    ) -> PrismResult<Self> {
        if amount.is_zero() {
            return Err(PrismError::ZeroAmount);
        }

        let escrow_id = EscrowId::derive(depositor, beneficiary, depositor_nonce, memo_hash);
        Ok(Self {
            network_id,
            escrow_id,
            depositor,
            beneficiary,
            asset,
            amount,
            depositor_nonce,
            expires_at_epoch,
            memo_hash,
            settlement_path,
        })
    }

    pub fn memo_hash(text: &str) -> Digest {
        Digest::domain_hash("prism-memo-v1", &[text.as_bytes()])
    }

    pub fn terms_hash(&self) -> PrismResult<Digest> {
        Digest::from_serializable("escrow-terms-wire-v2", self)
    }

    fn authorization_view(&self) -> EscrowAuthorizationView {
        EscrowAuthorizationView {
            network_id: self.network_id,
            escrow_id: self.escrow_id,
            depositor: self.depositor,
            beneficiary: self.beneficiary,
            asset: self.asset,
            amount: self.amount,
            depositor_nonce: self.depositor_nonce,
            expires_at_epoch: self.expires_at_epoch,
            memo_hash: self.memo_hash,
            settlement_path: self.settlement_path,
        }
    }
}

impl EscrowRelease {
    pub fn new(
        network_id: u32,
        escrow_id: EscrowId,
        beneficiary: AccountId,
        release_nonce: u64,
        observed_terms_hash: Digest,
    ) -> Self {
        Self {
            network_id,
            escrow_id,
            beneficiary,
            release_nonce,
            observed_terms_hash,
        }
    }

    fn authorization_view(&self) -> ReleaseAuthorizationView {
        ReleaseAuthorizationView {
            network_id: self.network_id,
            escrow_id: self.escrow_id,
            beneficiary: self.beneficiary,
            release_nonce: self.release_nonce,
            observed_terms_hash: self.observed_terms_hash,
        }
    }
}

impl SignedEscrowOpen {
    pub fn sign(terms: EscrowTerms, key_pair: &KeyPair) -> PrismResult<Self> {
        let depositor = key_pair.public_identity();
        if depositor.account != terms.depositor {
            return Err(PrismError::UnauthorizedDepositor {
                expected: terms.depositor,
                received: depositor.account,
            });
        }

        let signature = key_pair.sign(OPEN_DOMAIN, &terms.authorization_view())?;
        Ok(Self {
            depositor,
            terms,
            signature,
        })
    }

    pub fn verify(&self) -> PrismResult<()> {
        if self.depositor.account != self.terms.depositor {
            return Err(PrismError::UnauthorizedDepositor {
                expected: self.terms.depositor,
                received: self.depositor.account,
            });
        }

        verify(
            self.depositor,
            self.signature,
            OPEN_DOMAIN,
            &self.terms.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> PrismResult<TxId> {
        TxId::from_serializable("signed-escrow-open-v2", self)
    }
}

impl SignedEscrowRelease {
    pub fn sign(release: EscrowRelease, key_pair: &KeyPair) -> PrismResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != release.beneficiary {
            return Err(PrismError::UnauthorizedRelease {
                expected: release.beneficiary,
                received: signer.account,
            });
        }

        let signature = key_pair.sign(RELEASE_DOMAIN, &release.authorization_view())?;
        Ok(Self {
            signer,
            release,
            signature,
        })
    }

    pub fn verify(&self) -> PrismResult<()> {
        if self.signer.account != self.release.beneficiary {
            return Err(PrismError::UnauthorizedRelease {
                expected: self.release.beneficiary,
                received: self.signer.account,
            });
        }

        verify(
            self.signer,
            self.signature,
            RELEASE_DOMAIN,
            &self.release.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> PrismResult<TxId> {
        TxId::from_serializable("signed-escrow-release-v2", self)
    }
}
