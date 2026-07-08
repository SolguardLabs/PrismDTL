import assert from "node:assert/strict";
import test from "node:test";

import { readText } from "../helpers/scenario_helpers.js";

test("package expone scripts de test Bun desde el directorio tests", () => {
    const pkg = JSON.parse(readText("package.json"));
    assert.equal(pkg.name, "prism-dtl");
    assert.equal(pkg.private, true);
    assert.equal(pkg.type, "module");
    assert.equal(pkg.scripts.test, "bun test --timeout 30000 ./tests/node");
    assert.match(pkg.scripts["test:all"], /cargo test/u);
    assert.match(pkg.scripts["test:all"], /cargo clippy/u);
    assert.match(pkg.scripts["test:all"], /bun test --timeout 30000 \.\/tests\/node/u);
});

test("package mantiene las dependencias acotadas a tooling de desarrollo", () => {
    const pkg = JSON.parse(readText("package.json"));
    assert.equal(pkg.dependencies, undefined);
    assert.deepEqual(pkg.devDependencies, {
        prettier: "^3.8.3",
    });
});

test("la metadata Cargo usa la identidad del crate Prism", () => {
    const cargoToml = readText("Cargo.toml");
    assert.match(cargoToml, /name = "prism_dtl"/u);
    assert.match(cargoToml, /edition = "2024"/u);
    assert.match(cargoToml, /rust-version = "1\.96"/u);
    assert.match(cargoToml, /publish = false/u);
});

test("el manifiesto Cargo mantiene las dependencias esperadas del protocolo", () => {
    const cargoToml = readText("Cargo.toml");
    for (const name of ["blake3", "ed25519-dalek", "serde", "serde_json", "thiserror"]) {
        assert.match(cargoToml, new RegExp(`^${name} =`, "mu"));
    }
});

test("README presenta los comandos operativos", () => {
    const readme = readText("README.md");
    assert.match(readme, /^# Prism DTL/mu);
    assert.match(readme, /cargo fmt --all -- --check/u);
    assert.match(readme, /cargo test/u);
    assert.match(readme, /cargo clippy --all-targets --all-features --locked -- -D warnings/u);
});

test("README describe la estructura pública de módulos", () => {
    const readme = readText("README.md");
    for (const module of ["amount", "ids", "crypto", "escrow", "ledger", "error"]) {
        assert.match(readme, new RegExp(`- \`${module}\`:`, "u"));
    }
});

test("gitignore excluye salidas generadas de build", () => {
    const gitignore = readText(".gitignore");
    assert.match(gitignore, /\/target\//u);
    assert.match(gitignore, /\*\.log/u);
    assert.match(gitignore, /\*\.pdb/u);
});
