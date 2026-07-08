import assert from "node:assert/strict";
import test from "node:test";

import { assertHex32, scenario } from "../helpers/scenario_helpers.js";

test("los escenarios correctos comparten el mismo contrato superior", () => {
    for (const name of ["direct-release", "routed-release"]) {
        const payload = scenario(name);
        for (const key of [
            "schema_version",
            "scenario",
            "ok",
            "network_id",
            "asset",
            "identities",
            "escrow",
            "tx_ids",
            "balances",
            "nonces",
            "state_digest",
            "total_supply",
            "conservation_ok",
        ]) {
            assert.ok(Object.hasOwn(payload, key), `${name} no incluye ${key}`);
        }
    }
});

test("los balances de escenarios correctos se serializan como números", () => {
    const payload = scenario("direct-release");
    for (const value of Object.values(payload.balances)) {
        assert.equal(typeof value, "number");
    }
});

test("las cuentas de identidad son únicas dentro de un escenario", () => {
    const payload = scenario("routed-release");
    const accounts = Object.values(payload.identities).map((identity) => identity.account);
    assert.equal(new Set(accounts).size, accounts.length);
});

test("las claves públicas son únicas dentro de un escenario", () => {
    const payload = scenario("routed-release");
    const keys = Object.values(payload.identities).map((identity) => identity.public_key);
    assert.equal(new Set(keys).size, keys.length);
});

test("el id de escrow y el hash de términos no son intercambiables", () => {
    const payload = scenario("direct-release");
    assertHex32(payload.escrow.id);
    assertHex32(payload.escrow.terms_hash);
    assert.notEqual(payload.escrow.id, payload.escrow.terms_hash);
});

test("los escenarios de error usan un envelope de error estable", () => {
    const payload = scenario("wrong-network-open");
    assert.equal(payload.schema_version, "prism-scenario-v1");
    assert.equal(payload.ok, false);
    assert.equal(typeof payload.error_kind, "string");
    assert.equal(typeof payload.error, "string");
});

test("la red y el activo permanecen estables entre escenarios correctos", () => {
    const direct = scenario("direct-release");
    const routed = scenario("routed-release");
    assert.equal(direct.network_id, routed.network_id);
    assert.equal(direct.asset, routed.asset);
});

test("los escenarios rechazados preservan conservación cuando existe estado de ledger", () => {
    for (const name of ["stale-open-nonce", "stale-release-terms", "route-value-limit"]) {
        const payload = scenario(name);
        assert.equal(payload.conservation_ok, true);
        assert.equal(payload.total_supply, 1_000);
    }
});
