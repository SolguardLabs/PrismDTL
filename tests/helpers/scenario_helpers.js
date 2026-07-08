import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
export const root = join(here, "..", "..");
const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
const cache = new Map();

export function scenario(name) {
    if (!cache.has(name)) {
        const stdout = execFileSync(
            cargo,
            ["run", "--quiet", "--example", "scenario_vectors", "--", name],
            {
                cwd: root,
                encoding: "utf8",
                timeout: 120_000,
                env: { ...process.env, CARGO_TERM_COLOR: "never" },
            },
        );
        cache.set(name, JSON.parse(stdout));
    }
    return cache.get(name);
}

export function readText(path) {
    return readFileSync(join(root, path), "utf8");
}

export function assertHex32(value) {
    assert.equal(typeof value, "string");
    assert.match(value, /^[0-9a-f]{64}$/u);
}

export function assertScenarioHeader(payload, name, ok) {
    assert.equal(payload.schema_version, "prism-scenario-v1");
    assert.equal(payload.scenario, name);
    assert.equal(payload.ok, ok);
}

export function assertConserved(payload, totalSupply = 1_000) {
    assert.equal(payload.conservation_ok, true);
    assert.equal(payload.total_supply, totalSupply);
}

export function assertBalances(payload, balances) {
    assert.deepEqual(payload.balances, balances);
}

export function assertKnownParties(payload) {
    for (const key of ["depositor", "beneficiary", "operator", "integrator"]) {
        assertHex32(payload.identities[key].account);
        assertHex32(payload.identities[key].public_key);
    }
}

export function assertRejected(payload, name, kind) {
    assertScenarioHeader(payload, name, false);
    assert.equal(payload.error_kind, kind);
    assert.equal(typeof payload.error, "string");
    assert.ok(payload.error.length > 0);
}
