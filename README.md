[![CI](https://github.com/Pro-Kla/sabrix-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/Pro-Kla/sabrix-bench/actions)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/Pro-Kla/sabrix-bench)
[![Rust: 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

# sabrix-bench ⚡

> Ultra-fast, zero-bloat developer CLI and benchmark harness for Model Context Protocol (MCP) traffic inspection and agent loop latency profiling.

---

## 🎯 The Problem

Developers building autonomous AI agents locally lack visibility into raw **Model Context Protocol (MCP)** JSON-RPC tool traffic and have no lightweight way to measure per-turn serialization and proxy latency overhead.

Legacy approaches introduce massive performance and security taxes:
- **Legacy Python / Node Proxies:** Add **30ms – 50ms** of serialization and runtime tax per tool call.
- **SaaS AI Firewalls:** Incur **100ms – 250ms** of WAN network latency, TLS handshakes, and third-party cloud data egress per turn.
- Over a **30-turn agent loop**, legacy firewalls add **3.6+ seconds of dead wait time** and leak raw database queries and internal system commands outside your perimeter.

`sabrix-bench` gives you real-time visibility into your local MCP tool calls and benchmarks your agent loops in **microseconds ($< 2\ \mu\text{s}$)** with **zero network egress**.

---

## 🚀 Quickstart & Installation

### 1-Line Install (via Cargo & GitHub)

```bash
cargo install --git https://github.com/Pro-Kla/sabrix-bench
```

### Or Build & Install from Local Source

```bash
# Clone the repository and install the binary
git clone https://github.com/Pro-Kla/sabrix-bench.git
cd sabrix-bench
cargo install --path .
```

Or run directly with Cargo:

```bash
cargo run -- --help
```

---

## 🛠️ CLI Usage & Subcommands

### 1. `sabrix-bench trace` — Real-Time MCP Security Inspector

Inspect JSON-RPC 2.0 requests (`tools/call`, `resources/read`, `tools/list`) and flag dangerous tool mutations (destructive shell commands, SQL injections, leaked credentials).

#### Run Built-in Demo Scenarios:
```bash
sabrix-bench trace --demo
```

#### Inspect from Inline JSON Payload:
```bash
sabrix-bench trace -p '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "database_query",
    "arguments": { "sql": "DROP TABLE users; --" }
  }
}'
```

#### Pipe from Stdin:
```bash
cat mcp_message.json | sabrix-bench trace
```

---

### 2. `sabrix-bench bench` — Multi-Turn Agent Loop Benchmark

Simulate multi-turn autonomous agent loops to measure local parsing overhead, memory serialization, and latency percentiles ($p50$, $p95$, $p99$).

```bash
# Run a 30-turn benchmark with real-time progress
sabrix-bench bench --turns 30

# Benchmark larger context payloads (10x scale)
sabrix-bench bench --turns 50 --scale 10

# Output machine-readable JSON for CI/CD pipelines
sabrix-bench bench --turns 20 --json
```

---

### 3. `sabrix-bench compare` — Architectural Overhead Matrix

Output a side-by-side architectural comparison contrasting In-VPC embedded engines against legacy proxies and remote SaaS firewalls.

```bash
sabrix-bench compare
```

---

## 📊 Benchmark & Latency Tax Breakdown

| Layer / Architecture | Per-Turn Latency | 30-Turn Loop Delay | Egress & Privacy | Memory Footprint |
| :--- | :--- | :--- | :--- | :--- |
| **Sabrix In-VPC Engine** | **`< 2 µs`** (0.002 ms) | **`< 0.06 ms`** | **100% In-VPC (Zero Egress)** | **`< 15 MB`** |
| **Legacy Python / Node Proxy** | `35.0 ms` | `+1.05 seconds` | Local Cluster | `150 MB – 400 MB` |
| **SaaS AI Firewall** | `120.0 ms` | `+3.60 seconds` | Full Payload Egress | N/A (Cloud SaaS) |

---

## 🛡️ Built-in Risk Checks

`sabrix-bench` evaluates deterministic security rules locally in sub-microsecond time:
- **`MCP-SEC-001`**: Destructive Filesystem Operations (`rm -rf`, `mkfs`, `dd`, `chmod 777`)
- **`MCP-SEC-002`**: Remote Code Execution / Reverse Shells (`curl | bash`, `nc -e`, `/dev/tcp/`)
- **`MCP-SEC-003`**: Destructive SQL Mutations & Injections (`DROP TABLE`, `TRUNCATE`, `DELETE FROM`, `WHERE 1=1`)
- **`MCP-SEC-004-007`**: Leaked API Keys & Secrets (OpenAI `sk-`, GitHub `ghp_`, AWS `AKIA`, PEM Private Keys)
- **`MCP-SEC-008`**: Sensitive Path Egress (`/etc/passwd`, `~/.ssh/id_rsa`, `~/.aws/credentials`, `.env`)
- **`MCP-SEC-009`**: Unconstrained Arbitrary Execution Tool Invocations

---

## 🌟 Deploying to Production?

Enforce zero-egress In-VPC MCP security with millisecond-grade deterministic policy control:

👉 **[Deploy Sabrix In-VPC Gateway](https://sabrix.ai)**

---

## 📜 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
