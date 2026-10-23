use prism_dtl::{
    Amount, AssetId, BondedRebatePolicy, EscrowRelease, EscrowTerms, KeyPair, PrismLedger,
    SettlementPath, SignedEscrowOpen, SignedEscrowRelease, quote_bonded_rebate,
};

#[test]
fn bonded_route_reconciles_liquid_escrow_and_operator_collateral() {
    let depositor = KeyPair::from_seed([41; 32]);
    let beneficiary = KeyPair::from_seed([42; 32]);
    let operator = KeyPair::from_seed([43; 32]);
    let mut ledger = PrismLedger::new(77, AssetId::native());
    for identity in [&depositor, &beneficiary, &operator] {
        ledger.register_account(identity.public_identity()).unwrap();
    }
    ledger
        .credit_genesis(
            depositor.public_identity().account,
            Amount::new(1_000).unwrap(),
        )
        .unwrap();
    ledger
        .credit_genesis(
            operator.public_identity().account,
            Amount::new(500).unwrap(),
        )
        .unwrap();
    ledger
        .post_operator_bond(
            operator.public_identity().account,
            Amount::new(500).unwrap(),
        )
        .unwrap();
    ledger
        .set_bonded_rebate_policy(
            operator.public_identity().account,
            BondedRebatePolicy {
                routing_epoch: 7,
                maximum_rebate: Amount::new(400).unwrap(),
            },
        )
        .unwrap();
    let mut path = SettlementPath::direct(operator.public_identity().account);
    path.routing_epoch = 7;
    let terms = EscrowTerms::new(
        77,
        depositor.public_identity().account,
        beneficiary.public_identity().account,
        AssetId::native(),
        Amount::new(1_000).unwrap(),
        0,
        100,
        EscrowTerms::memo_hash("bonded-settlement"),
        path,
    )
    .unwrap();
    let signed_open = SignedEscrowOpen::sign(terms, &depositor).unwrap();
    ledger.open_escrow(&signed_open).unwrap();
    let quote = ledger
        .bonded_rebate_quote(terms.escrow_id)
        .unwrap()
        .unwrap();
    assert_eq!(quote.rebate.units(), 144);
    let release = EscrowRelease::new(
        77,
        terms.escrow_id,
        beneficiary.public_identity().account,
        0,
        terms.terms_hash().unwrap(),
    );
    ledger
        .release_escrow(&SignedEscrowRelease::sign(release, &beneficiary).unwrap())
        .unwrap();
    assert_eq!(
        ledger
            .balance_of(beneficiary.public_identity().account)
            .unwrap()
            .units(),
        1_144
    );
    assert_eq!(
        ledger
            .operator_bond_of(operator.public_identity().account)
            .unwrap()
            .units(),
        356
    );
    assert_eq!(ledger.supply_snapshot().unwrap().bonded.units(), 356);
    ledger.verify_conservation().unwrap();
}

#[test]
fn rebate_curve_is_monotone_in_eligible_depth() {
    let policy = BondedRebatePolicy {
        routing_epoch: 7,
        maximum_rebate: Amount::new(400).unwrap(),
    };
    let shallow = quote_bonded_rebate(
        policy,
        Amount::new(500).unwrap(),
        Amount::new(1_000).unwrap(),
    )
    .unwrap();
    let deep = quote_bonded_rebate(
        policy,
        Amount::new(1_500).unwrap(),
        Amount::new(1_000).unwrap(),
    )
    .unwrap();
    assert!(deep.rebate > shallow.rebate);
    assert!(deep.rebate <= policy.maximum_rebate);
}
