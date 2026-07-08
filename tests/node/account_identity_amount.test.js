import assert from "node:assert/strict";
import test from "node:test";

import { assertRejected, scenario } from "../helpers/scenario_helpers.js";

test("rechaza el registro duplicado de cuentas", () => {
    const payload = scenario("duplicate-account-registration");
    assertRejected(payload, "duplicate-account-registration", "AccountAlreadyExists");
    assert.equal(payload.balances.depositor, 1_000);
});

test("exige consistencia de identidad al registrar cuentas", () => {
    const payload = scenario("identity-consistency");
    assertRejected(payload, "identity-consistency", "IdentityMismatch");
});

test("rechaza términos con importe cero", () => {
    const payload = scenario("zero-amount-terms");
    assertRejected(payload, "zero-amount-terms", "ZeroAmount");
});

test("mantiene acotado el rango de importes", () => {
    const payload = scenario("amount-range");
    assertRejected(payload, "amount-range", "AmountOutOfRange");
});

test("devuelve un fallo estructurado para escenarios desconocidos", () => {
    const payload = scenario("unknown-scenario-name");
    assertRejected(payload, "unknown-scenario-name", "UnknownScenario");
});

test("el rechazo por rango de importe no expone estado del ledger", () => {
    const payload = scenario("amount-range");
    assert.equal(payload.balances, undefined);
    assert.equal(payload.state_digest, undefined);
});

test("el rechazo por importe cero ocurre antes de cambiar estado de apertura", () => {
    const payload = scenario("zero-amount-terms");
    assert.equal(payload.balances, undefined);
    assert.equal(payload.nonces, undefined);
});
