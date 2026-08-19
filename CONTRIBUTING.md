# Contributing to sabrix-bench

Thank you for your interest in contributing to `sabrix-bench`! We welcome contributions from systems engineers, AI developers, and security researchers.

`sabrix-bench` is built to give the AI engineering community an ultra-fast, zero-bloat utility to inspect raw Model Context Protocol (MCP) JSON-RPC tool traffic and benchmark per-turn agent loop latency.

---

## 🛠️ Development Setup

### Prerequisites
- Rust 1.70 or higher (`rustup update stable`)
- Git

### Build & Run
```bash
# Clone the repository
git clone https://github.com/sabrix-ai/sabrix-bench.git
cd sabrix-bench

# Build in debug mode
cargo build

# Run test suite
cargo test

# Run code style & linter checks
cargo fmt --check
cargo clippy -- -D warnings
```

---

## 🛡️ Adding New MCP Security Rules

All deterministic inspection rules live inside [`src/inspector.rs`](src/inspector.rs).

To add a new security rule:
1. Identify the rule class and assign an ID (e.g., `MCP-SEC-010`).
2. Add signature pattern checks inside `McpInspector::evaluate_security_rules`.
3. Set the appropriate `RiskLevel` (`Low`, `Medium`, `High`, `Critical`).
4. Add unit test assertions in `tests` module in [`src/inspector.rs`](src/inspector.rs) verifying that malicious tool calls trigger the rule and safe tool calls pass.

Example rule addition:
```rust
// Rule 10: Docker daemon socket mounting
if text_lower.contains("/var/run/docker.sock") {
    findings.push(RiskFinding {
        rule_id: "MCP-SEC-010".to_string(),
        level: RiskLevel::Critical,
        title: "Docker Socket Mount Probe".to_string(),
        details: "Tool payload attempts to bind or read host Docker daemon socket".to_string(),
        matched_snippet: "/var/run/docker.sock".to_string(),
    });
}
```

---

## 🧪 Verification & Pre-Commit Checks

Before opening a pull request, run the unified verification script:

```bash
./verify.sh
```

This verifies:
1. Formatting: `cargo fmt --check`
2. Static Analysis: `cargo clippy -- -D warnings`
3. Unit & Integration Tests: `cargo test --all`
4. CLI Subcommand Executions: `trace --demo`, `bench --turns 30`, `compare`

---

## 📜 Pull Request Guidelines

1. **Keep Changes Focused**: Each PR should address a single feature, bug fix, or rule expansion.
2. **Zero Proprietary Dependencies**: Dependencies must come directly from crates.io; no relative path crates.
3. **Commit Messages**: Use Conventional Commits (`feat: ...`, `fix: ...`, `docs: ...`, `perf: ...`).
4. **License Agreement**: By contributing, you agree that your contributions will be licensed under both the Apache-2.0 and MIT licenses.
