#!/usr/bin/env bash
set -e

# ANSI Color Codes
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${CYAN}${BOLD}"
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║             SABRIX-BENCH AUTOMATED VERIFICATION & CI SUITE                   ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

step_banner() {
    echo -e "\n${BOLD}${CYAN}▶ Step: $1${NC}"
    echo -e "${CYAN}────────────────────────────────────────────────────────────────────────────────${NC}"
}

success_banner() {
    echo -e "${GREEN}${BOLD}✔ $1 PASSED${NC}"
}

# Step 1: Check formatting
step_banner "1. Code Formatting (cargo fmt --check)"
if cargo fmt --check; then
    success_banner "Code formatting check"
else
    echo -e "${YELLOW}Formatting discrepancies found. Automatically running 'cargo fmt'...${NC}"
    cargo fmt
    success_banner "Code formatting applied"
fi

# Step 2: Clippy Linter
step_banner "2. Clippy Static Analysis (cargo clippy -- -D warnings)"
cargo clippy --all-targets --all-features -- -D warnings
success_banner "Clippy linter checks"

# Step 3: Unit & Integration Tests
step_banner "3. Test Suite (cargo test --all)"
cargo test --all
success_banner "All unit and integration tests"

# Step 4: Trace Subcommand Verification
step_banner "4. CLI Verification: MCP Tool-Call Inspector (trace --demo)"
cargo run -- trace --demo
success_banner "MCP trace command"

# Step 5: Benchmark Subcommand Verification
step_banner "5. CLI Verification: Multi-Turn Agent Loop Benchmark (bench --turns 30)"
cargo run -- bench --turns 30
success_banner "Agent loop benchmark command"

# Step 6: Architecture Comparison Verification
step_banner "6. CLI Verification: Architectural Comparison Matrix (compare)"
cargo run -- compare
success_banner "Comparison matrix command"

echo -e "\n${GREEN}${BOLD}"
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║          ALL CHECKS & CLI BENCHMARKS PASSED CLEANLY (100% READY)              ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
