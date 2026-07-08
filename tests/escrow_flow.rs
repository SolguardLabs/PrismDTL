use prism_dtl::{
    Amount, AssetId, Digest, EscrowRelease, EscrowTerms, KeyPair, PrismError, PrismLedger,
    SettlementPath, SignedEscrowOpen, SignedEscrowRelease,
};

struct SettlementEnvironment {
    ledger: PrismLedger,
    depositor: KeyPair,
    beneficiary: KeyPair,
    operator: KeyPair,
    integrator: KeyPair,
    asset: AssetId,
    network_id: u32,
}

fn settlement_environment(initial_deposit: u128) -> SettlementEnvironment {
    let network_id = 77;
    let asset = AssetId::native();
    let depositor = KeyPair::from_seed([1u8; 32]);
    let beneficiary = KeyPair::from_seed([2u8; 32]);
    let operator = KeyPair::from_seed([3u8; 32]);
    let integrator = KeyPair::from_seed([4u8; 32]);
    let mut ledger = PrismLedger::new(network_id, asset);

    ledger
        .register_account(depositor.public_identity())
        .unwrap();
    ledger
        .register_account(beneficiary.public_identity())
        .unwrap();
    ledger.register_account(operator.public_identity()).unwrap();
    ledger
        .register_account(integrator.public_identity())
        .unwrap();
    ledger
        .credit_genesis(
            depositor.public_identity().account,
            Amount::new(initial_deposit).unwrap(),
        )
        .unwrap();

    SettlementEnvironment {
        ledger,
        depositor,
        beneficiary,
        operator,
        integrator,
        asset,
        network_id,
    }
}

fn open_terms(fx: &SettlementEnvironment, amount: u128, fee: u128) -> EscrowTerms {
    let depositor_id = fx.depositor.public_identity().account;
    let beneficiary_id = fx.beneficiary.public_identity().account;
    let operator_id = fx.operator.public_identity().account;
    let memo_hash = EscrowTerms::memo_hash("purchase-order:42");

    EscrowTerms::new(
        fx.network_id,
        depositor_id,
        beneficiary_id,
        fx.asset,
        Amount::new(amount).unwrap(),
        fx.ledger.next_nonce(depositor_id).unwrap(),
        100,
        memo_hash,
        SettlementPath {
            operator: operator_id,
            operator_fee: Amount::new(fee).unwrap(),
            rebate_recipient: None,
            rebate_amount: Amount::zero(),
            routing_epoch: 1,
        },
    )
    .unwrap()
}

#[test]
fn honest_escrow_release_preserves_supply() {
    let mut fx = settlement_environment(1_000);
    let terms = open_terms(&fx, 250, 10);
    let open = SignedEscrowOpen::sign(terms, &fx.depositor).unwrap();
    let escrow_id = open.terms.escrow_id;

    fx.ledger.open_escrow(&open).unwrap();
    let release = EscrowRelease::new(
        fx.network_id,
        escrow_id,
        fx.beneficiary.public_identity().account,
        fx.ledger
            .release_nonce(fx.beneficiary.public_identity().account)
            .unwrap(),
        open.terms.terms_hash().unwrap(),
    );
    let signed_release = SignedEscrowRelease::sign(release, &fx.beneficiary).unwrap();
    fx.ledger.release_escrow(&signed_release).unwrap();

    assert_eq!(
        fx.ledger
            .balance_of(fx.depositor.public_identity().account)
            .unwrap(),
        Amount::new(750).unwrap()
    );
    assert_eq!(
        fx.ledger
            .balance_of(fx.beneficiary.public_identity().account)
            .unwrap(),
        Amount::new(240).unwrap()
    );
    assert_eq!(
        fx.ledger
            .balance_of(fx.operator.public_identity().account)
            .unwrap(),
        Amount::new(10).unwrap()
    );
    assert_eq!(fx.ledger.total_supply(), Amount::new(1_000).unwrap());
    fx.ledger.verify_conservation().unwrap();
}

fn routed_terms(fx: &SettlementEnvironment, amount: u128, fee: u128, rebate: u128) -> EscrowTerms {
    let depositor_id = fx.depositor.public_identity().account;
    let beneficiary_id = fx.beneficiary.public_identity().account;
    let operator_id = fx.operator.public_identity().account;
    let memo_hash = EscrowTerms::memo_hash("rfq:usdc-route:2026-07-08");

    EscrowTerms::new(
        fx.network_id,
        depositor_id,
        beneficiary_id,
        fx.asset,
        Amount::new(amount).unwrap(),
        fx.ledger.next_nonce(depositor_id).unwrap(),
        100,
        memo_hash,
        SettlementPath {
            operator: operator_id,
            operator_fee: Amount::new(fee).unwrap(),
            rebate_recipient: Some(fx.integrator.public_identity().account),
            rebate_amount: Amount::new(rebate).unwrap(),
            routing_epoch: 2,
        },
    )
    .unwrap()
}

#[test]
fn routed_escrow_release_preserves_supply() {
    let mut fx = settlement_environment(1_000);
    let terms = routed_terms(&fx, 600, 10, 40);
    let open = SignedEscrowOpen::sign(terms, &fx.depositor).unwrap();
    let escrow_id = open.terms.escrow_id;

    fx.ledger.open_escrow(&open).unwrap();
    let release = EscrowRelease::new(
        fx.network_id,
        escrow_id,
        fx.beneficiary.public_identity().account,
        fx.ledger
            .release_nonce(fx.beneficiary.public_identity().account)
            .unwrap(),
        open.terms.terms_hash().unwrap(),
    );
    let signed_release = SignedEscrowRelease::sign(release, &fx.beneficiary).unwrap();

    fx.ledger.release_escrow(&signed_release).unwrap();

    assert_eq!(
        fx.ledger
            .balance_of(fx.depositor.public_identity().account)
            .unwrap(),
        Amount::new(400).unwrap()
    );
    assert_eq!(
        fx.ledger
            .balance_of(fx.beneficiary.public_identity().account)
            .unwrap(),
        Amount::new(550).unwrap()
    );
    assert_eq!(
        fx.ledger
            .balance_of(fx.operator.public_identity().account)
            .unwrap(),
        Amount::new(10).unwrap()
    );
    assert_eq!(
        fx.ledger
            .balance_of(fx.integrator.public_identity().account)
            .unwrap(),
        Amount::new(40).unwrap()
    );
    assert_eq!(fx.ledger.total_supply(), Amount::new(1_000).unwrap());
    fx.ledger.verify_conservation().unwrap();
}

#[test]
fn release_requires_current_terms_hash() {
    let mut fx = settlement_environment(1_000);
    let terms = open_terms(&fx, 250, 10);
    let open = SignedEscrowOpen::sign(terms, &fx.depositor).unwrap();
    let escrow_id = open.terms.escrow_id;
    fx.ledger.open_escrow(&open).unwrap();

    let release = EscrowRelease::new(
        fx.network_id,
        escrow_id,
        fx.beneficiary.public_identity().account,
        fx.ledger
            .release_nonce(fx.beneficiary.public_identity().account)
            .unwrap(),
        Digest::zero(),
    );
    let signed_release = SignedEscrowRelease::sign(release, &fx.beneficiary).unwrap();

    assert!(matches!(
        fx.ledger.release_escrow(&signed_release),
        Err(PrismError::TermsHashMismatch { .. })
    ));
}

#[test]
fn tampered_amount_rejects_open_signature() {
    let mut fx = settlement_environment(1_000);
    let mut terms = open_terms(&fx, 250, 10);
    let open = SignedEscrowOpen::sign(terms, &fx.depositor).unwrap();
    terms.amount = Amount::new(251).unwrap();

    let tampered = SignedEscrowOpen { terms, ..open };

    assert_eq!(
        fx.ledger.open_escrow(&tampered),
        Err(PrismError::InvalidSignature)
    );
}

#[test]
fn replayed_release_is_rejected() {
    let mut fx = settlement_environment(1_000);
    let terms = open_terms(&fx, 250, 10);
    let open = SignedEscrowOpen::sign(terms, &fx.depositor).unwrap();
    let escrow_id = open.terms.escrow_id;
    fx.ledger.open_escrow(&open).unwrap();

    let release = EscrowRelease::new(
        fx.network_id,
        escrow_id,
        fx.beneficiary.public_identity().account,
        fx.ledger
            .release_nonce(fx.beneficiary.public_identity().account)
            .unwrap(),
        open.terms.terms_hash().unwrap(),
    );
    let signed_release = SignedEscrowRelease::sign(release, &fx.beneficiary).unwrap();
    fx.ledger.release_escrow(&signed_release).unwrap();

    assert_eq!(
        fx.ledger.release_escrow(&signed_release),
        Err(PrismError::EscrowReleased(escrow_id))
    );
}

#[test]
fn escrow_open_checks_depositor_nonce() {
    let mut fx = settlement_environment(1_000);
    let terms = open_terms(&fx, 100, 0);
    let first = SignedEscrowOpen::sign(terms, &fx.depositor).unwrap();
    fx.ledger.open_escrow(&first).unwrap();

    let second_terms = EscrowTerms::new(
        fx.network_id,
        fx.depositor.public_identity().account,
        fx.beneficiary.public_identity().account,
        fx.asset,
        Amount::new(100).unwrap(),
        0,
        100,
        EscrowTerms::memo_hash("purchase-order:43"),
        SettlementPath::direct(fx.operator.public_identity().account),
    )
    .unwrap();
    let second = SignedEscrowOpen::sign(second_terms, &fx.depositor).unwrap();

    assert!(matches!(
        fx.ledger.open_escrow(&second),
        Err(PrismError::NonceMismatch {
            expected: 1,
            received: 0,
            ..
        })
    ));
}
