#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/tooling.sh
. "${SCRIPT_DIR}/tooling.sh"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

require_tool cargo

cargo test --locked
run_bun test --timeout 30000 ./tests/node
