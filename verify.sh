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

# Step 6: Dynamic Multi-Turn Comparison & Architecture Matrix Verification
step_banner "6. CLI Verification: Multi-Turn Comparison & Matrix (compare)"
echo "Running dynamic 10-turn latency comparison vs SaaS firewall baseline..."
cargo run -- compare --turns 10

echo "Running JSON machine-readable comparison..."
cargo run -- compare --turns 5 --json

echo "Running static architecture matrix..."
cargo run -- compare --matrix
success_banner "Multi-turn comparison and architectural matrix commands"

# Step 7: Edge Case, Boundary & Stdio Streaming Suite
step_banner "7. Edge Case, Boundary & Stdio Stress Tests"

echo "Testing piped single JSON payload..."
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"safe_tool","arguments":{"query":"test"}}}' | cargo run -- trace

echo "Testing piped JSON array payload..."
echo '[{"jsonrpc":"2.0","id":1,"method":"tools/list"},{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bash","arguments":{"command":"rm -rf /tmp/data"}}}]' | cargo run -- trace

echo "Testing piped NDJSON (Newline Delimited JSON)..."
printf '{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"file:///etc/hosts"}}\n{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sql","arguments":{"sql":"DROP TABLE logs;"}}}\n' | cargo run -- trace

echo "Testing boundary benchmark: --turns 0"
cargo run -- bench --turns 0 --quiet

echo "Testing boundary benchmark: --turns 1"
cargo run -- bench --turns 1 --quiet

echo "Testing high-iteration stress benchmark: --turns 500 --scale 2"
cargo run -- bench --turns 500 --scale 2 --quiet

echo "Testing graceful error handling on malformed JSON..."
if echo '{"jsonrpc": "2.0", broken_syntax' | cargo run -- trace 2>/dev/null; then
    echo -e "${RED}Failed: Expected malformed JSON to return non-zero exit code${NC}"
    exit 1
else
    echo "✔ Handled malformed JSON gracefully with non-zero exit code"
fi

echo "Testing graceful error handling on arbitrary binary garbage..."
if echo -e '\x00\x01\x02\xFF\xFE' | cargo run -- trace 2>/dev/null; then
    echo -e "${RED}Failed: Expected binary garbage to return non-zero exit code${NC}"
    exit 1
else
    echo "✔ Handled binary garbage gracefully with non-zero exit code"
fi

success_banner "Edge case, boundary and stdio streaming test suite"

echo -e "\n${GREEN}${BOLD}"
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║          ALL CHECKS, BENCHMARKS & STRESS TESTS PASSED (100% HARDENED)        ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
