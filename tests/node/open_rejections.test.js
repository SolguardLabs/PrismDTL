import assert from "node:assert/strict";
import test from "node:test";

import {
    assertBalances,
    assertConserved,
    assertRejected,
    scenario,
} from "../helpers/scenario_helpers.js";

test("rechaza un nonce de apertura antiguo", () => {
    const payload = scenario("stale-open-nonce");
    assertRejected(payload, "stale-open-nonce", "NonceMismatch");
    assert.equal(payload.nonces.depositor, 1);
    assertConserved(payload);
});

test("un nonce de apertura antiguo conserva el estado de escrow aceptado previamente", () => {
    const payload = scenario("stale-open-nonce");
    assertBalances(payload, {
        depositor: 900,
        beneficiary: 0,
        operator: 0,
        integrator: 0,
    });
});

test("un importe de apertura ajustado se rechaza por verificación de firma", () => {
    const payload = scenario("tampered-open-amount");
    assertRejected(payload, "tampered-open-amount", "InvalidSignature");
    assertBalances(payload, {
        depositor: 1_000,
        beneficiary: 0,
        operator: 0,
        integrator: 0,
    });
});

test("una apertura sin balance suficiente no puede debitar al depositante", () => {
    const payload = scenario("overdraw-open");
    assertRejected(payload, "overdraw-open", "InsufficientFunds");
    assert.equal(payload.balances.depositor, 1_000);
    assert.equal(payload.nonces.depositor, 0);
});

test("un destinatario de rebate desconocido se rechaza antes del débito", () => {
    const payload = scenario("unknown-rebate-recipient");
    assertRejected(payload, "unknown-rebate-recipient", "AccountNotFound");
    assert.equal(payload.balances.depositor, 1_000);
    assert.equal(payload.nonces.depositor, 0);
});

test("la solicitud de apertura debe apuntar a la red local", () => {
    const payload = scenario("wrong-network-open");
    assertRejected(payload, "wrong-network-open", "Policy");
    assert.equal(payload.balances.depositor, 1_000);
    assert.equal(payload.nonces.depositor, 0);
});

test("las respuestas de rechazo de apertura incluyen compromisos de estado", () => {
    for (const name of ["overdraw-open", "wrong-network-open", "unknown-rebate-recipient"]) {
        const payload = scenario(name);
        assert.equal(typeof payload.state_digest, "string");
        assert.equal(payload.state_digest.length, 64);
    }
});
