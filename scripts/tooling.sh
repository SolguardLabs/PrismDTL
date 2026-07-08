#!/usr/bin/env bash
set -euo pipefail

# Utilidades compartidas para ejecutar el toolchain local desde Bash, Git Bash o
# WSL sin depender de que todos los perfiles de shell carguen la misma ruta.

if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    . "${HOME}/.cargo/env"
fi

if [[ -d "${HOME}/.cargo/bin" ]]; then
    export PATH="${HOME}/.cargo/bin:${PATH}"
fi

require_tool() {
    local tool="$1"

    if command -v "${tool}" >/dev/null 2>&1; then
        return 0
    fi

    printf 'No se encontró %s en el PATH de Bash.\n' "${tool}" >&2
    return 127
}

ensure_rust_components() {
    if command -v rustup >/dev/null 2>&1; then
        rustup component add rustfmt clippy >/dev/null
    fi
}

run_bun() {
    if command -v bun >/dev/null 2>&1 && bun --version >/dev/null 2>&1; then
        bun "$@"
        return 0
    fi

    if grep -qi microsoft /proc/version 2>/dev/null && command -v cmd.exe >/dev/null 2>&1; then
        if cmd.exe /D /C bun --version >/dev/null 2>&1; then
            cmd.exe /D /C bun "$@"
            return 0
        fi
    fi

    cat >&2 <<'EOF'
No se pudo ejecutar Bun desde Bash.
Instala Bun en este entorno, o ejecuta desde PowerShell:
  bun run test:all
EOF
    return 127
}
