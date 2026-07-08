import assert from "node:assert/strict";
import test from "node:test";

import {
    assertBalances,
    assertConserved,
    assertRejected,
    scenario,
} from "../helpers/scenario_helpers.js";

test("rechaza una liberación con hash de términos antiguo", () => {
    const payload = scenario("stale-release-terms");
    assertRejected(payload, "stale-release-terms", "TermsHashMismatch");
    assert.equal(payload.nonces.beneficiary_release, 0);
    assertConserved(payload);
});

test("el rechazo por términos antiguos deja bloqueado el valor del escrow", () => {
    const payload = scenario("stale-release-terms");
    assertBalances(payload, {
        depositor: 750,
        beneficiary: 0,
        operator: 0,
        integrator: 0,
    });
});

test("un escrow ya liberado no puede liberarse dos veces", () => {
    const payload = scenario("replayed-release");
    assertRejected(payload, "replayed-release", "EscrowReleased");
    assert.equal(payload.nonces.beneficiary_release, 1);
});

test("una liberación repetida conserva los balances de la primera liberación", () => {
    const payload = scenario("replayed-release");
    assertBalances(payload, {
        depositor: 750,
        beneficiary: 240,
        operator: 10,
        integrator: 0,
    });
    assertConserved(payload);
});

test("los pagos de ruta no pueden superar el importe del escrow", () => {
    const payload = scenario("route-value-limit");
    assertRejected(payload, "route-value-limit", "Policy");
    assert.equal(payload.balances.depositor, 900);
    assert.equal(payload.nonces.beneficiary_release, 0);
    assertConserved(payload);
});

test("la solicitud de liberación debe apuntar a la red local", () => {
    const payload = scenario("wrong-network-release");
    assertRejected(payload, "wrong-network-release", "Policy");
    assert.equal(payload.balances.depositor, 750);
    assert.equal(payload.nonces.beneficiary_release, 0);
    assertConserved(payload);
});

test("las respuestas de rechazo de liberación incluyen compromisos de estado", () => {
    for (const name of ["stale-release-terms", "replayed-release", "wrong-network-release"]) {
        const payload = scenario(name);
        assert.equal(typeof payload.state_digest, "string");
        assert.equal(payload.state_digest.length, 64);
    }
});
