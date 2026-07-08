#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/tooling.sh
. "${SCRIPT_DIR}/tooling.sh"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

require_tool cargo
ensure_rust_components

cargo fmt --all -- --check
cargo build --all-targets --locked
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings

run_bun install --frozen-lockfile
run_bun run fmt:check
run_bun run build
run_bun test ./tests/node
