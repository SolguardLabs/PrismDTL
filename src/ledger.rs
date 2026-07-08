use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AccountId, Amount, AssetId, Digest, EscrowId, PrismError, PrismResult, PublicIdentity,
    SignedEscrowOpen, SignedEscrowRelease, TxId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountState {
    pub id: AccountId,
    pub identity: PublicIdentity,
    pub next_nonce: u64,
    pub balance: Amount,
    pub release_nonce: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EscrowState {
    pub id: EscrowId,
    pub terms: crate::EscrowTerms,
    pub released: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum JournalOp {
    FundingCredit {
        account: AccountId,
        amount: Amount,
    },
    EscrowDebit {
        account: AccountId,
        escrow_id: EscrowId,
        amount: Amount,
    },
    EscrowCredit {
        account: AccountId,
        escrow_id: EscrowId,
        amount: Amount,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub tx_id: TxId,
    pub op: JournalOp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PrismLedger {
    network_id: u32,
    asset: AssetId,
    accounts: BTreeMap<AccountId, AccountState>,
    escrows: BTreeMap<EscrowId, EscrowState>,
    seen_transactions: BTreeSet<TxId>,
    total_supply: Amount,
    journal: Vec<JournalEntry>,
}

impl PrismLedger {
    pub fn new(network_id: u32, asset: AssetId) -> Self {
        Self {
            network_id,
            asset,
            accounts: BTreeMap::new(),
            escrows: BTreeMap::new(),
            seen_transactions: BTreeSet::new(),
            total_supply: Amount::zero(),
            journal: Vec::new(),
        }
    }

    pub fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn asset(&self) -> AssetId {
        self.asset
    }

    pub fn register_account(&mut self, identity: PublicIdentity) -> PrismResult<()> {
        identity.verify_consistency()?;

        if self.accounts.contains_key(&identity.account) {
            return Err(PrismError::AccountAlreadyExists(identity.account));
        }

        self.accounts.insert(
            identity.account,
            AccountState {
                id: identity.account,
                identity,
                next_nonce: 0,
                balance: Amount::zero(),
                release_nonce: 0,
            },
        );
        Ok(())
    }

    pub fn credit_genesis(&mut self, account: AccountId, amount: Amount) -> PrismResult<()> {
        self.ensure_account(account)?;
        let mut candidate = self.clone();
        let current = candidate.balance_of(account)?;
        let next = current.checked_add(amount)?;
        candidate
            .accounts
            .get_mut(&account)
            .ok_or(PrismError::AccountNotFound(account))?
            .balance = next;
        candidate.total_supply = candidate.total_supply.checked_add(amount)?;
        let tx_id = TxId::from_serializable(
            "prism-genesis-credit-v1",
            &(account, amount, self.journal.len()),
        )?;
        candidate.journal.push(JournalEntry {
            tx_id,
            op: JournalOp::FundingCredit { account, amount },
        });
        candidate.verify_conservation()?;
        *self = candidate;
        Ok(())
    }

    pub fn balance_of(&self, account: AccountId) -> PrismResult<Amount> {
        Ok(self.ensure_account(account)?.balance)
    }

    pub fn next_nonce(&self, account: AccountId) -> PrismResult<u64> {
        Ok(self.ensure_account(account)?.next_nonce)
    }

    pub fn release_nonce(&self, account: AccountId) -> PrismResult<u64> {
        Ok(self.ensure_account(account)?.release_nonce)
    }

    pub fn total_supply(&self) -> Amount {
        self.total_supply
    }

    pub fn escrow(&self, escrow_id: EscrowId) -> PrismResult<&EscrowState> {
        self.escrows
            .get(&escrow_id)
            .ok_or(PrismError::EscrowNotFound(escrow_id))
    }

    pub fn open_escrow(&mut self, open: &SignedEscrowOpen) -> PrismResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.open_escrow_inner(open)?;
        candidate.verify_conservation()?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn release_escrow(&mut self, release: &SignedEscrowRelease) -> PrismResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.release_escrow_inner(release)?;
        candidate.verify_conservation()?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn state_digest(&self) -> PrismResult<Digest> {
        Digest::from_serializable(
            "prism-ledger-state-v1",
            &(
                self.network_id,
                self.asset,
                &self.accounts,
                &self.escrows,
                &self.seen_transactions,
                self.total_supply,
            ),
        )
    }

    fn open_escrow_inner(&mut self, open: &SignedEscrowOpen) -> PrismResult<TxId> {
        open.verify()?;
        let terms = open.terms;

        if terms.network_id != self.network_id {
            return Err(PrismError::Policy("wrong network".to_owned()));
        }

        if terms.asset != self.asset {
            return Err(PrismError::AssetMismatch {
                expected: self.asset,
                received: terms.asset,
            });
        }

        if self.escrows.contains_key(&terms.escrow_id) {
            return Err(PrismError::EscrowAlreadyExists(terms.escrow_id));
        }

        self.ensure_account(terms.beneficiary)?;
        self.ensure_account(terms.settlement_path.operator)?;
        if let Some(rebate_recipient) = terms.settlement_path.rebate_recipient {
            self.ensure_account(rebate_recipient)?;
        }
        let depositor_state = self.ensure_account(terms.depositor)?;
        if depositor_state.next_nonce != terms.depositor_nonce {
            return Err(PrismError::NonceMismatch {
                account: terms.depositor,
                expected: depositor_state.next_nonce,
                received: terms.depositor_nonce,
            });
        }

        if depositor_state.balance < terms.amount {
            return Err(PrismError::InsufficientFunds {
                account: terms.depositor,
                available: depositor_state.balance,
                required: terms.amount,
            });
        }

        let tx_id = open.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(PrismError::DuplicateTransaction(tx_id));
        }

        let next_balance = depositor_state.balance.checked_sub(terms.amount)?;
        let depositor = self
            .accounts
            .get_mut(&terms.depositor)
            .ok_or(PrismError::AccountNotFound(terms.depositor))?;
        depositor.balance = next_balance;
        depositor.next_nonce = depositor
            .next_nonce
            .checked_add(1)
            .ok_or(PrismError::NonceOverflow)?;

        self.escrows.insert(
            terms.escrow_id,
            EscrowState {
                id: terms.escrow_id,
                terms,
                released: false,
            },
        );
        self.seen_transactions.insert(tx_id);
        self.journal.push(JournalEntry {
            tx_id,
            op: JournalOp::EscrowDebit {
                account: terms.depositor,
                escrow_id: terms.escrow_id,
                amount: terms.amount,
            },
        });

        Ok(tx_id)
    }

    fn release_escrow_inner(&mut self, signed: &SignedEscrowRelease) -> PrismResult<TxId> {
        signed.verify()?;

        if signed.release.network_id != self.network_id {
            return Err(PrismError::Policy("wrong network".to_owned()));
        }

        let escrow = self.escrow(signed.release.escrow_id)?.clone();
        if escrow.released {
            return Err(PrismError::EscrowReleased(escrow.id));
        }

        if signed.signer.account != escrow.terms.beneficiary {
            return Err(PrismError::UnauthorizedRelease {
                expected: escrow.terms.beneficiary,
                received: signed.signer.account,
            });
        }

        let expected_terms_hash = escrow.terms.terms_hash()?;
        if signed.release.observed_terms_hash != expected_terms_hash {
            return Err(PrismError::TermsHashMismatch {
                escrow_id: escrow.id,
                expected: expected_terms_hash,
                received: signed.release.observed_terms_hash,
            });
        }

        let expected_release_nonce = self.ensure_account(signed.signer.account)?.release_nonce;
        if expected_release_nonce != signed.release.release_nonce {
            return Err(PrismError::NonceMismatch {
                account: signed.signer.account,
                expected: expected_release_nonce,
                received: signed.release.release_nonce,
            });
        }

        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(PrismError::DuplicateTransaction(tx_id));
        }

        let path = escrow.terms.settlement_path;
        let operator_fee = path.operator_fee;
        let rebate_amount = path.rebate_amount;
        let payable_total = operator_fee.checked_add(rebate_amount)?;
        if payable_total > escrow.terms.amount {
            return Err(PrismError::Policy(
                "settlement path exceeds escrow value".to_owned(),
            ));
        }

        let beneficiary_amount = escrow.terms.amount.checked_sub(payable_total)?;
        let rebate_recipient = path.rebate_recipient.unwrap_or(escrow.terms.beneficiary);

        self.credit(escrow.terms.beneficiary, beneficiary_amount)?;
        self.credit(path.operator, operator_fee)?;
        self.credit(rebate_recipient, rebate_amount)?;

        self.escrows
            .get_mut(&escrow.id)
            .ok_or(PrismError::EscrowNotFound(escrow.id))?
            .released = true;
        self.accounts
            .get_mut(&signed.signer.account)
            .ok_or(PrismError::AccountNotFound(signed.signer.account))?
            .release_nonce = expected_release_nonce
            .checked_add(1)
            .ok_or(PrismError::NonceOverflow)?;
        self.seen_transactions.insert(tx_id);
        self.journal.push(JournalEntry {
            tx_id,
            op: JournalOp::EscrowCredit {
                account: escrow.terms.beneficiary,
                escrow_id: escrow.id,
                amount: escrow.terms.amount,
            },
        });

        Ok(tx_id)
    }

    fn credit(&mut self, account: AccountId, amount: Amount) -> PrismResult<()> {
        self.ensure_account(account)?;
        let current = self.balance_of(account)?;
        self.accounts
            .get_mut(&account)
            .ok_or(PrismError::AccountNotFound(account))?
            .balance = current.checked_add(amount)?;
        Ok(())
    }

    fn ensure_account(&self, account: AccountId) -> PrismResult<&AccountState> {
        self.accounts
            .get(&account)
            .ok_or(PrismError::AccountNotFound(account))
    }

    pub fn verify_conservation(&self) -> PrismResult<()> {
        let mut liquid = Amount::zero();
        for account in self.accounts.values() {
            liquid = liquid.checked_add(account.balance)?;
        }

        let mut locked = Amount::zero();
        for escrow in self.escrows.values() {
            if !escrow.released {
                locked = locked.checked_add(escrow.terms.amount)?;
            }
        }

        let observed = liquid.checked_add(locked)?;
        if observed != self.total_supply {
            return Err(PrismError::Conservation {
                asset: self.asset,
                expected: self.total_supply,
                observed,
            });
        }

        Ok(())
    }
}
