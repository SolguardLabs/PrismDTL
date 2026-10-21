use prism_dtl::{
    AccountId, Amount, AssetId, Digest, EscrowRelease, EscrowTerms, KeyPair, PrismError,
    PrismLedger, PrismResult, SettlementPath, SignedEscrowOpen, SignedEscrowRelease,
};
use serde_json::{Value, json};

struct Environment {
    ledger: PrismLedger,
    depositor: KeyPair,
    beneficiary: KeyPair,
    operator: KeyPair,
    integrator: KeyPair,
    asset: AssetId,
    network_id: u32,
}

impl Environment {
    fn new(initial_deposit: u128) -> PrismResult<Self> {
        let network_id = 77;
        let asset = AssetId::native();
        let depositor = KeyPair::from_seed([1u8; 32]);
        let beneficiary = KeyPair::from_seed([2u8; 32]);
        let operator = KeyPair::from_seed([3u8; 32]);
        let integrator = KeyPair::from_seed([4u8; 32]);
        let mut ledger = PrismLedger::new(network_id, asset);

        ledger.register_account(depositor.public_identity())?;
        ledger.register_account(beneficiary.public_identity())?;
        ledger.register_account(operator.public_identity())?;
        ledger.register_account(integrator.public_identity())?;
        ledger.credit_genesis(
            depositor.public_identity().account,
            Amount::new(initial_deposit)?,
        )?;

        Ok(Self {
            ledger,
            depositor,
            beneficiary,
            operator,
            integrator,
            asset,
            network_id,
        })
    }
}

fn main() {
    let scenario = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "direct-release".to_owned());

    let payload = match scenario.as_str() {
        "direct-release" => direct_release(),
        "routed-release" => routed_release(),
        "lifecycle-digests" => lifecycle_digests(),
        "stale-open-nonce" => stale_open_nonce(),
        "tampered-open-amount" => tampered_open_amount(),
        "stale-release-terms" => stale_release_terms(),
        "replayed-release" => replayed_release(),
        "overdraw-open" => overdraw_open(),
        "route-value-limit" => route_value_limit(),
        "unknown-rebate-recipient" => unknown_rebate_recipient(),
        "wrong-network-open" => wrong_network_open(),
        "wrong-network-release" => wrong_network_release(),
        "duplicate-account-registration" => duplicate_account_registration(),
        "zero-amount-terms" => zero_amount_terms(),
        "identity-consistency" => identity_consistency(),
        "amount-range" => amount_range(),
        other => Ok(json!({
            "schema_version": "prism-scenario-v1",
            "scenario": other,
            "ok": false,
            "error_kind": "UnknownScenario",
            "error": format!("unknown scenario: {other}")
        })),
    };

    match payload {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value).unwrap());
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&error_payload("internal", &error)).unwrap()
            );
            std::process::exit(1);
        }
    }
}

fn direct_terms(
    env: &Environment,
    amount: u128,
    fee: u128,
    memo: &str,
) -> PrismResult<EscrowTerms> {
    EscrowTerms::new(
        env.network_id,
        env.depositor.public_identity().account,
        env.beneficiary.public_identity().account,
        env.asset,
        Amount::new(amount)?,
        env.ledger
            .next_nonce(env.depositor.public_identity().account)?,
        100,
        EscrowTerms::memo_hash(memo),
        SettlementPath {
            operator: env.operator.public_identity().account,
            operator_fee: Amount::new(fee)?,
            rebate_recipient: None,
            rebate_amount: Amount::zero(),
            routing_epoch: 1,
        },
    )
}

fn routed_terms(
    env: &Environment,
    amount: u128,
    fee: u128,
    rebate: u128,
    memo: &str,
) -> PrismResult<EscrowTerms> {
    EscrowTerms::new(
        env.network_id,
        env.depositor.public_identity().account,
        env.beneficiary.public_identity().account,
        env.asset,
        Amount::new(amount)?,
        env.ledger
            .next_nonce(env.depositor.public_identity().account)?,
        144,
        EscrowTerms::memo_hash(memo),
        SettlementPath {
            operator: env.operator.public_identity().account,
            operator_fee: Amount::new(fee)?,
            rebate_recipient: Some(env.integrator.public_identity().account),
            rebate_amount: Amount::new(rebate)?,
            routing_epoch: 2,
        },
    )
}

fn signed_release(
    env: &Environment,
    escrow_id: prism_dtl::EscrowId,
    observed_terms_hash: Digest,
    network_id: u32,
) -> PrismResult<SignedEscrowRelease> {
    let beneficiary = env.beneficiary.public_identity().account;
    let release = EscrowRelease::new(
        network_id,
        escrow_id,
        beneficiary,
        env.ledger.release_nonce(beneficiary)?,
        observed_terms_hash,
    );
    SignedEscrowRelease::sign(release, &env.beneficiary)
}

fn direct_release() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = direct_terms(&env, 250, 10, "purchase-order:42")?;
    let terms_hash = terms.terms_hash()?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    let open_tx = env.ledger.open_escrow(&open)?;
    let release = signed_release(&env, terms.escrow_id, terms_hash, env.network_id)?;
    let release_tx = env.ledger.release_escrow(&release)?;

    success_payload(
        "direct-release",
        &env,
        &terms,
        json!({ "open": open_tx, "release": release_tx }),
    )
}

fn routed_release() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = routed_terms(&env, 600, 10, 40, "rfq:usdc-route:2026-07-08")?;
    let terms_hash = terms.terms_hash()?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    let open_tx = env.ledger.open_escrow(&open)?;
    let release = signed_release(&env, terms.escrow_id, terms_hash, env.network_id)?;
    let release_tx = env.ledger.release_escrow(&release)?;

    success_payload(
        "routed-release",
        &env,
        &terms,
        json!({ "open": open_tx, "release": release_tx }),
    )
}

fn lifecycle_digests() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let initial = env.ledger.state_digest()?;
    let terms = direct_terms(&env, 125, 5, "digest-probe")?;
    let terms_hash = terms.terms_hash()?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    env.ledger.open_escrow(&open)?;
    let after_open = env.ledger.state_digest()?;
    let release = signed_release(&env, terms.escrow_id, terms_hash, env.network_id)?;
    env.ledger.release_escrow(&release)?;
    let after_release = env.ledger.state_digest()?;

    Ok(json!({
        "schema_version": "prism-scenario-v1",
        "scenario": "lifecycle-digests",
        "ok": true,
        "digests": {
            "initial": initial,
            "after_open": after_open,
            "after_release": after_release
        },
        "balances": balances(&env)?,
        "total_supply": env.ledger.total_supply(),
        "conservation_ok": env.ledger.verify_conservation().is_ok()
    }))
}

fn stale_open_nonce() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let first_terms = direct_terms(&env, 100, 0, "nonce:first")?;
    let first = SignedEscrowOpen::sign(first_terms, &env.depositor)?;
    env.ledger.open_escrow(&first)?;

    let second_terms = EscrowTerms::new(
        env.network_id,
        env.depositor.public_identity().account,
        env.beneficiary.public_identity().account,
        env.asset,
        Amount::new(100)?,
        0,
        100,
        EscrowTerms::memo_hash("nonce:second"),
        SettlementPath::direct(env.operator.public_identity().account),
    )?;
    let second = SignedEscrowOpen::sign(second_terms, &env.depositor)?;

    let result = env.ledger.open_escrow(&second);
    rejected_payload("stale-open-nonce", &env, result)
}

fn tampered_open_amount() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let mut terms = direct_terms(&env, 250, 10, "integrity:amount")?;
    let signed = SignedEscrowOpen::sign(terms, &env.depositor)?;
    terms.amount = Amount::new(251)?;
    let adjusted = SignedEscrowOpen { terms, ..signed };

    let result = env.ledger.open_escrow(&adjusted);
    rejected_payload("tampered-open-amount", &env, result)
}

fn stale_release_terms() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = direct_terms(&env, 250, 10, "release:stale-terms")?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    env.ledger.open_escrow(&open)?;
    let release = signed_release(&env, terms.escrow_id, Digest::zero(), env.network_id)?;

    let result = env.ledger.release_escrow(&release);
    rejected_payload("stale-release-terms", &env, result)
}

fn replayed_release() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = direct_terms(&env, 250, 10, "release:replay")?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    env.ledger.open_escrow(&open)?;
    let release = signed_release(&env, terms.escrow_id, terms.terms_hash()?, env.network_id)?;
    env.ledger.release_escrow(&release)?;

    let result = env.ledger.release_escrow(&release);
    rejected_payload("replayed-release", &env, result)
}

fn overdraw_open() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = direct_terms(&env, 1_001, 0, "open:overdraw")?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;

    let result = env.ledger.open_escrow(&open);
    rejected_payload("overdraw-open", &env, result)
}

fn route_value_limit() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = routed_terms(&env, 100, 80, 30, "route:value-limit")?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    let result = env.ledger.open_escrow(&open);
    rejected_payload("route-value-limit", &env, result)
}

fn unknown_rebate_recipient() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = EscrowTerms::new(
        env.network_id,
        env.depositor.public_identity().account,
        env.beneficiary.public_identity().account,
        env.asset,
        Amount::new(250)?,
        env.ledger
            .next_nonce(env.depositor.public_identity().account)?,
        144,
        EscrowTerms::memo_hash("route:unknown-rebate"),
        SettlementPath {
            operator: env.operator.public_identity().account,
            operator_fee: Amount::new(5)?,
            rebate_recipient: Some(AccountId::from_bytes([9u8; 32])),
            rebate_amount: Amount::new(10)?,
            routing_epoch: 2,
        },
    )?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;

    let result = env.ledger.open_escrow(&open);
    rejected_payload("unknown-rebate-recipient", &env, result)
}

fn wrong_network_open() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = EscrowTerms::new(
        env.network_id + 1,
        env.depositor.public_identity().account,
        env.beneficiary.public_identity().account,
        env.asset,
        Amount::new(250)?,
        env.ledger
            .next_nonce(env.depositor.public_identity().account)?,
        100,
        EscrowTerms::memo_hash("open:wrong-network"),
        SettlementPath::direct(env.operator.public_identity().account),
    )?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;

    let result = env.ledger.open_escrow(&open);
    rejected_payload("wrong-network-open", &env, result)
}

fn wrong_network_release() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let terms = direct_terms(&env, 250, 10, "release:wrong-network")?;
    let open = SignedEscrowOpen::sign(terms, &env.depositor)?;
    env.ledger.open_escrow(&open)?;
    let release = signed_release(
        &env,
        terms.escrow_id,
        terms.terms_hash()?,
        env.network_id + 1,
    )?;

    let result = env.ledger.release_escrow(&release);
    rejected_payload("wrong-network-release", &env, result)
}

fn duplicate_account_registration() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let identity = env.operator.public_identity();
    let result = env.ledger.register_account(identity);
    rejected_payload("duplicate-account-registration", &env, result)
}

fn zero_amount_terms() -> PrismResult<Value> {
    let env = Environment::new(1_000)?;
    let result = EscrowTerms::new(
        env.network_id,
        env.depositor.public_identity().account,
        env.beneficiary.public_identity().account,
        env.asset,
        Amount::zero(),
        env.ledger
            .next_nonce(env.depositor.public_identity().account)?,
        100,
        EscrowTerms::memo_hash("open:zero-amount"),
        SettlementPath::direct(env.operator.public_identity().account),
    );

    match result {
        Ok(_) => Ok(json!({
            "schema_version": "prism-scenario-v1",
            "scenario": "zero-amount-terms",
            "ok": true
        })),
        Err(error) => Ok(error_payload("zero-amount-terms", &error)),
    }
}

fn identity_consistency() -> PrismResult<Value> {
    let mut env = Environment::new(1_000)?;
    let mut identity = env.depositor.public_identity();
    identity.public_key = env.operator.public_identity().public_key;

    let result = env.ledger.register_account(identity);
    rejected_payload("identity-consistency", &env, result)
}

fn amount_range() -> PrismResult<Value> {
    let result = Amount::new(10_000_000_000_001);
    match result {
        Ok(_) => Ok(json!({
            "schema_version": "prism-scenario-v1",
            "scenario": "amount-range",
            "ok": true
        })),
        Err(error) => Ok(error_payload("amount-range", &error)),
    }
}

fn success_payload(
    scenario: &str,
    env: &Environment,
    terms: &EscrowTerms,
    tx_ids: Value,
) -> PrismResult<Value> {
    Ok(json!({
        "schema_version": "prism-scenario-v1",
        "scenario": scenario,
        "ok": true,
        "network_id": env.ledger.network_id(),
        "asset": env.ledger.asset(),
        "identities": identities(env),
        "escrow": {
            "id": terms.escrow_id,
            "released": env.ledger.escrow(terms.escrow_id)?.released,
            "terms_hash": terms.terms_hash()?,
            "amount": terms.amount,
            "route": terms.settlement_path
        },
        "tx_ids": tx_ids,
        "balances": balances(env)?,
        "nonces": nonces(env)?,
        "state_digest": env.ledger.state_digest()?,
        "total_supply": env.ledger.total_supply(),
        "conservation_ok": env.ledger.verify_conservation().is_ok()
    }))
}

fn rejected_payload<T>(
    scenario: &str,
    env: &Environment,
    result: PrismResult<T>,
) -> PrismResult<Value> {
    match result {
        Ok(_) => Ok(json!({
            "schema_version": "prism-scenario-v1",
            "scenario": scenario,
            "ok": true,
            "balances": balances(env)?,
            "nonces": nonces(env)?,
            "state_digest": env.ledger.state_digest()?,
            "total_supply": env.ledger.total_supply(),
            "conservation_ok": env.ledger.verify_conservation().is_ok()
        })),
        Err(error) => {
            let mut payload = error_payload(scenario, &error);
            payload["balances"] = balances(env)?;
            payload["nonces"] = nonces(env)?;
            payload["state_digest"] = json!(env.ledger.state_digest()?);
            payload["total_supply"] = json!(env.ledger.total_supply());
            payload["conservation_ok"] = json!(env.ledger.verify_conservation().is_ok());
            Ok(payload)
        }
    }
}

fn balances(env: &Environment) -> PrismResult<Value> {
    Ok(json!({
        "depositor": env.ledger.balance_of(env.depositor.public_identity().account)?.units(),
        "beneficiary": env.ledger.balance_of(env.beneficiary.public_identity().account)?.units(),
        "operator": env.ledger.balance_of(env.operator.public_identity().account)?.units(),
        "integrator": env.ledger.balance_of(env.integrator.public_identity().account)?.units()
    }))
}

fn nonces(env: &Environment) -> PrismResult<Value> {
    Ok(json!({
        "depositor": env.ledger.next_nonce(env.depositor.public_identity().account)?,
        "beneficiary_release": env.ledger.release_nonce(env.beneficiary.public_identity().account)?
    }))
}

fn identities(env: &Environment) -> Value {
    json!({
        "depositor": env.depositor.public_identity(),
        "beneficiary": env.beneficiary.public_identity(),
        "operator": env.operator.public_identity(),
        "integrator": env.integrator.public_identity()
    })
}

fn error_payload(scenario: &str, error: &PrismError) -> Value {
    json!({
        "schema_version": "prism-scenario-v1",
        "scenario": scenario,
        "ok": false,
        "error_kind": error_kind(error),
        "error": error.to_string()
    })
}

fn error_kind(error: &PrismError) -> &'static str {
    match error {
        PrismError::AmountOverflow => "AmountOverflow",
        PrismError::AmountUnderflow => "AmountUnderflow",
        PrismError::AmountOutOfRange { .. } => "AmountOutOfRange",
        PrismError::ZeroAmount => "ZeroAmount",
        PrismError::InvalidPublicKey => "InvalidPublicKey",
        PrismError::InvalidSignature => "InvalidSignature",
        PrismError::Serialization(_) => "Serialization",
        PrismError::IdentityMismatch(_) => "IdentityMismatch",
        PrismError::AccountNotFound(_) => "AccountNotFound",
        PrismError::AccountAlreadyExists(_) => "AccountAlreadyExists",
        PrismError::EscrowNotFound(_) => "EscrowNotFound",
        PrismError::EscrowAlreadyExists(_) => "EscrowAlreadyExists",
        PrismError::EscrowReleased(_) => "EscrowReleased",
        PrismError::UnauthorizedDepositor { .. } => "UnauthorizedDepositor",
        PrismError::UnauthorizedRelease { .. } => "UnauthorizedRelease",
        PrismError::NonceMismatch { .. } => "NonceMismatch",
        PrismError::NonceOverflow => "NonceOverflow",
        PrismError::DuplicateTransaction(_) => "DuplicateTransaction",
        PrismError::InsufficientFunds { .. } => "InsufficientFunds",
        PrismError::AssetMismatch { .. } => "AssetMismatch",
        PrismError::TermsHashMismatch { .. } => "TermsHashMismatch",
        PrismError::Policy(_) => "Policy",
        PrismError::Conservation { .. } => "Conservation",
    }
}
