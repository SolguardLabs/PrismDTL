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

test("la liquidación enrutada devuelve el envelope esperado", () => {
    const payload = scenario("routed-release");
    assertScenarioHeader(payload, "routed-release", true);
    assert.equal(payload.escrow.released, true);
    assert.equal(payload.escrow.amount, 600);
    assertConserved(payload);
});

test("la liquidación enrutada serializa todas las identidades participantes", () => {
    const payload = scenario("routed-release");
    assertKnownParties(payload);
});

test("la liquidación enrutada aplica comisión de operador y rebate de integrador", () => {
    const payload = scenario("routed-release");
    assertBalances(payload, {
        depositor: 400,
        beneficiary: 550,
        operator: 10,
        integrator: 40,
    });
});

test("la liquidación enrutada serializa la metadata de ruta", () => {
    const payload = scenario("routed-release");
    assert.equal(payload.escrow.route.operator_fee, 10);
    assert.equal(payload.escrow.route.rebate_amount, 40);
    assert.equal(payload.escrow.route.routing_epoch, 2);
    assertHex32(payload.escrow.route.operator);
    assertHex32(payload.escrow.route.rebate_recipient);
});

test("la ruta enrutada apunta a las cuentas de operador e integrador", () => {
    const payload = scenario("routed-release");
    assert.equal(payload.escrow.route.operator, payload.identities.operator.account);
    assert.equal(payload.escrow.route.rebate_recipient, payload.identities.integrator.account);
});

test("el beneficiario enrutado recibe el importe neto de comisión y rebate", () => {
    const payload = scenario("routed-release");
    const amount = payload.escrow.amount;
    const fee = payload.escrow.route.operator_fee;
    const rebate = payload.escrow.route.rebate_amount;
    assert.equal(payload.balances.beneficiary, amount - fee - rebate);
});

test("la liquidación enrutada no deja valor bloqueado tras la liberación", () => {
    const payload = scenario("routed-release");
    assert.equal(payload.escrow.released, true);
    assert.equal(payload.nonces.beneficiary_release, 1);
});
