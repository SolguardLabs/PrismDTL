import assert from "node:assert/strict";
import test from "node:test";

import {
    assertBalances,
    assertConserved,
    assertHex32,
    assertKnownParties,
    assertScenarioHeader,
    scenario,
} from "../helpers/scenario_helpers.js";

test("la liquidación directa devuelve el envelope esperado", () => {
    const payload = scenario("direct-release");
    assertScenarioHeader(payload, "direct-release", true);
    assert.equal(payload.network_id, 77);
    assertHex32(payload.asset);
    assertHex32(payload.state_digest);
    assert.equal(payload.escrow.released, true);
});

test("la liquidación directa serializa todas las identidades participantes", () => {
    const payload = scenario("direct-release");
    assertKnownParties(payload);
});

test("la liquidación directa distribuye el valor del escrow entre las partes", () => {
    const payload = scenario("direct-release");
    assertBalances(payload, {
        depositor: 750,
        beneficiary: 240,
        operator: 10,
        integrator: 0,
    });
    assertConserved(payload);
});

test("la liquidación directa avanza los nonces de depositante y liberación", () => {
    const payload = scenario("direct-release");
    assert.deepEqual(payload.nonces, {
        depositor: 1,
        beneficiary_release: 1,
    });
});

test("la liquidación directa incluye identificadores de transacción únicos", () => {
    const payload = scenario("direct-release");
    assertHex32(payload.tx_ids.open);
    assertHex32(payload.tx_ids.release);
    assert.notEqual(payload.tx_ids.open, payload.tx_ids.release);
});

test("la liquidación directa incluye hash de términos e id de escrow", () => {
    const payload = scenario("direct-release");
    assertHex32(payload.escrow.id);
    assertHex32(payload.escrow.terms_hash);
    assert.equal(payload.escrow.amount, 250);
});

test("la ruta directa no asigna rebate por defecto", () => {
    const payload = scenario("direct-release");
    assert.equal(payload.escrow.route.operator_fee, 10);
    assert.equal(payload.escrow.route.rebate_recipient, null);
    assert.equal(payload.escrow.route.rebate_amount, 0);
    assert.equal(payload.escrow.route.routing_epoch, 1);
});

test("la cuenta del operador directo coincide con el operador de ruta", () => {
    const payload = scenario("direct-release");
    assert.equal(payload.escrow.route.operator, payload.identities.operator.account);
});
