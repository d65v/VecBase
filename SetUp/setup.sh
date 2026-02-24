#!/usr/bin/env bash
# VecBase — setup.sh
# Installs system dependencies and verifies the Rust toolchain.
# Author: d65v <https://github.com/d65v>

set -euo pipefail

BOLD="\033[1m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

info()    { echo -e "${GREEN}[setup]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[warn] ${RESET} $*"; }
error()   { echo -e "${RED}[error]${RESET} $*"; exit 1; }

echo -e "${BOLD}VecBase Setup${RESET}"
echo "────────────────────────────────────"

# ── Rust ──────────────────────────────────────────────────────────────────────
if ! command -v rustc &>/dev/null; then
    warn "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
else
    info "Rust found: $(rustc --version)"
fi

if ! command -v cargo &>/dev/null; then
    error "cargo not found after rustup install. Please re-run or install manually."
fi

# Ensure nightly is not needed; stable is sufficient
RUST_VERSION=$(rustc --version | awk '{print $2}')
info "Cargo: $(cargo --version)"
info "Rust version: $RUST_VERSION"

# ── Components ────────────────────────────────────────────────────────────────
info "Installing rustfmt and clippy..."
rustup component add rustfmt clippy 2>/dev/null || true

# ── Data directory ────────────────────────────────────────────────────────────
info "Creating data directory..."
mkdir -p ./data

# ── .env ──────────────────────────────────────────────────────────────────────
if [ ! -f .env ]; then
    info "Creating .env from example..."
    cp vcore/src/plug-ins/env.example .env
else
    info ".env already exists, skipping."
fi

# ── Docker check (optional) ───────────────────────────────────────────────────
if command -v docker &>/dev/null; then
    info "Docker found: $(docker --version)"
else
    warn "Docker not found — skipping. Install Docker if you want container support."
fi

echo ""
echo -e "${GREEN}Setup complete!${RESET}"
echo ""
echo "Next steps:"
echo "  make build   — build the release binary"
echo "  make run     — run VecBase"
echo "  make test    — run tests"
echo ""
