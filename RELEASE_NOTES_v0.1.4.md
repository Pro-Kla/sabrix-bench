# Release v0.1.4: Dynamic Multi-Turn Latency Comparison Engine ⚡

`sabrix-bench` v0.1.4 is officially published on [Crates.io](https://crates.io/crates/sabrix-bench). This release upgrades the `compare` command into an empirical, live benchmarking engine that measures turn-by-turn in-process CPU execution and quantifies cumulative wall-clock latency saved vs. remote SaaS AI firewall WAN baselines (~120 ms).

---

## 🚀 Key Updates & Highlights

- **Dynamic Multi-Turn Comparison (`sabrix-bench compare`)**:
  - Added `--turns <N>` flag (default: `30`) to simulate custom multi-turn agent loops.
  - Measures live CPU evaluation overhead for each turn in safe Rust with `std::time::Instant`.
  - Contrasts in-process execution (~`2 µs` / `0.002 ms`) with remote SaaS firewall WAN latency (`120 ms`).
  - Displays turn-by-turn ASCII tables with exact per-turn latency and time-saved metrics.

- **Executive Summary Box**:
  - Highlights total in-process governance time (e.g. `0.058 ms`), total remote SaaS latency tax (e.g. `3,600.00 ms`), compounded net time saved (e.g. `+3.599 s`), speedup multiplier (`20,000x+`), and eliminated data egress.

- **Machine-Readable JSON Output (`--json`)**:
  - Supports `--json` flag on `compare` for CI/CD performance gates and automated benchmarking pipelines.

- **Static Architectural Matrix Mode (`--matrix`)**:
  - Preserved the comprehensive architectural matrix via `sabrix-bench compare --matrix`.

- **Comprehensive Test Suite & CI Hardening**:
  - Added integration test suite in `tests/compare_test.rs` validating default table rendering, custom turn counts, custom SaaS baseline latencies (`--saas-latency-ms`), JSON serialization schemas, and boundary conditions.
  - 100% clean passes across `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings`.

---

## 📦 Quickstart & Installation

### Install via Cargo (Crates.io)

```bash
cargo install sabrix-bench
```

### Or Build & Install from Source

```bash
git clone https://github.com/Pro-Kla/sabrix-bench.git
cd sabrix-bench
cargo install --path .
```

### CLI Invocations

```bash
# Run dynamic 30-turn live latency comparison
sabrix-bench compare --turns 30

# Output machine-readable JSON for CI/CD pipelines
sabrix-bench compare --turns 20 --json

# View full static architectural comparison matrix
sabrix-bench compare --matrix

# Run built-in MCP inspection demo across safe & malicious payloads
sabrix-bench trace --demo

# Run multi-turn agent loop percentile benchmark
sabrix-bench bench --turns 30
```

---

## 🧪 Verification & Test Suite

- **24/24 Tests Passing:** 20 unit tests + 4 integration tests covering security rules, flag transpositions, SQL shorthands, streaming NDJSON, boundary conditions, and the dynamic comparison engine.
- **Zero-Warning Guarantee:** Passes `cargo fmt --check` and `cargo clippy -- -D warnings`.

---

## 🛡️ Community & Governance

- Dual-licensed under **Apache-2.0** or **MIT** at your option.
- Contributions welcome! See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines on adding new MCP inspection rules.
- Production In-VPC Gateway: [https://sabrix.ai](https://sabrix.ai)
