# Release v0.1.0: MCP JSON-RPC Tracer & Agent Loop Latency Profiler ⚡

`sabrix-bench` is a lightweight, zero-bloat developer CLI and benchmark tool written in safe Rust for inspecting Model Context Protocol (MCP) JSON-RPC 2.0 tool traffic, catching dangerous execution mutations, and profiling turn-by-turn agent loop latency in microseconds.

---

## 🚀 Key Capabilities & Highlights

- **Sub-Microsecond Deterministic Security Inspection (`< 2 µs` rule evaluation):**
  - **Destructive Filesystem Guard (`MCP-SEC-001`):** Flags `rm -rf`, `rmdir /s`, `mkfs`, `dd if=`, and `chmod 777`.
  - **Remote Code Execution & Reverse Shells (`MCP-SEC-002`):** Intercepts `curl | bash`, `wget | sh`, `nc -e`, and `/dev/tcp/`.
  - **Destructive SQL & Schema Mutations (`MCP-SEC-003`):** Detects `DROP TABLE`, `TRUNCATE`, `DROP DATABASE`, and `WHERE 1=1` injections.
  - **Credential & Secret Leakage (`MCP-SEC-004-007`):** Unmasks exposed OpenAI keys (`sk-...`), GitHub tokens (`ghp_...`), AWS IAM credentials (`AKIA...`), and raw PEM private keys.
  - **Sensitive Path Protection (`MCP-SEC-008`):** Flags reads to `/etc/passwd`, `/etc/shadow`, `~/.ssh/id_rsa`, and `.env`.

- **Multi-Turn Autonomous Agent Loop Profiler:**
  - Simulates $N$-turn agent loops, recording turn-by-turn serialization, parsing, and inspection latency.
  - Computes complete percentile distributions: Min, Mean, Median ($p50$), $p95$, $p99$, Max, and Standard Deviation ($\sigma$).
  - Evaluates the multi-turn latency tax across 5, 10, 20, 30, and 50 turns.

- **Architecture Comparison Matrix (`sabrix-bench compare`):**
  - Contrasts In-Process Rust engines ($< 20\ \mu\text{s}$) against local HTTP proxies ($35\ \text{ms}$) and remote SaaS AI firewalls ($120\ \text{ms}$).
  - Demonstrates how In-VPC architectures eliminate $3.6+\text{s}$ of compounding latency and prevent cloud data egress in 30-turn agent loops.

- **Polymorphic Stdio Streaming & Zero-Panic Guarantee:**
  - Supports single JSON-RPC objects, JSON arrays (`[{...}, {...}]`), and NDJSON streaming streams from stdin.
  - Hardened against malformed inputs, truncated streams, and arbitrary binary data with clean terminal notices.

---

## 📦 Quickstart & Usage

### Installation

```bash
cargo install sabrix-bench
```

Or build from source:

```bash
git clone https://github.com/Pro-Kla/sabrix-bench.git
cd sabrix-bench
cargo build --release
```

### CLI Invocations

```bash
# Run built-in inspection demo across safe and malicious tool payloads
sabrix-bench trace --demo

# Inspect piped MCP JSON-RPC message from stdin
cat mcp_payload.json | sabrix-bench trace

# Benchmark a 30-turn agent loop
sabrix-bench bench --turns 30

# Output machine-readable JSON for CI/CD pipelines
sabrix-bench bench --turns 20 --json

# View architectural comparison matrix
sabrix-bench compare
```

---

## 🧪 Verification & Test Suite

- **13/13 Unit Tests Passing:** Covering deterministic security rules, multi-line SQL formatting, NDJSON streaming, boundary turns (`--turns 0`, `--turns 1`, `--turns 500`), and malformed payload error handling.
- **Cross-Platform CI Matrix:** Automated testing across Ubuntu, macOS, and Windows.
- **Zero-Warning Guarantee:** Passes `cargo fmt --check` and `cargo clippy -- -D warnings`.

---

## 🛡️ Community & Governance

- Dual-licensed under **Apache-2.0** or **MIT** at your option.
- Contributions welcome! See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines on adding new MCP inspection rules.
- Production In-VPC Gateway: [https://sabrix.ai](https://sabrix.ai)
