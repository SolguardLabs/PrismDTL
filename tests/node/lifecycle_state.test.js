import assert from "node:assert/strict";
import test from "node:test";

import {
    assertBalances,
    assertConserved,
    assertHex32,
    assertScenarioHeader,
    scenario,
} from "../helpers/scenario_helpers.js";

test("el escenario de digests de ciclo de vida devuelve el envelope esperado", () => {
    const payload = scenario("lifecycle-digests");
    assertScenarioHeader(payload, "lifecycle-digests", true);
    assert.equal(payload.conservation_ok, true);
});

test("los digests de ciclo de vida son compromisos hexadecimales de 32 bytes", () => {
    const payload = scenario("lifecycle-digests");
    assertHex32(payload.digests.initial);
    assertHex32(payload.digests.after_open);
    assertHex32(payload.digests.after_release);
});

test("los digests de ciclo de vida cambian en cada paso de liquidación", () => {
    const payload = scenario("lifecycle-digests");
    assert.notEqual(payload.digests.initial, payload.digests.after_open);
    assert.notEqual(payload.digests.after_open, payload.digests.after_release);
    assert.notEqual(payload.digests.initial, payload.digests.after_release);
});

test("el escenario de ciclo de vida conserva el suministro", () => {
    const payload = scenario("lifecycle-digests");
    assertBalances(payload, {
        depositor: 875,
        beneficiary: 120,
        operator: 5,
        integrator: 0,
    });
    assertConserved(payload);
});

test("los digests finales directo y enrutado son distintos", () => {
    const direct = scenario("direct-release");
    const routed = scenario("routed-release");
    assert.notEqual(direct.state_digest, routed.state_digest);
});

test("el mismo escenario emite un digest determinista estable", () => {
    const first = scenario("direct-release");
    const second = scenario("direct-release");
    assert.equal(first.state_digest, second.state_digest);
});
