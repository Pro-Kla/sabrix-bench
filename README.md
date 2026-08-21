[![Crates.io](https://img.shields.io/crates/v/sabrix-bench.svg)](https://crates.io/crates/sabrix-bench)
[![CI](https://github.com/Pro-Kla/sabrix-bench/actions/workflows/ci.yml/badge.svg)](https://github.com/Pro-Kla/sabrix-bench/actions)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/Pro-Kla/sabrix-bench)
[![Rust: 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

# sabrix-bench ⚡

> **The "wrk / hyperfine for AI proxies and LLM gateways"** — A fast, vendor-neutral benchmarking harness and Model Context Protocol (MCP) traffic inspector in pure safe Rust.

---

## 🎯 What is `sabrix-bench`?

`sabrix-bench` is an open-source, vendor-neutral benchmarking tool designed to measure pure client-visible HTTP & Server-Sent Events (SSE) streaming metrics against **any** AI proxy, firewall, or LLM gateway (LiteLLM, Cloudflare AI Gateway, Portkey, vLLM, or Sabrix).

Traditional web benchmark tools (wrk, vegeta) only measure total request duration ($t_{\text{total}}$), which is dominated by upstream LLM token generation. `sabrix-bench` inspects what users actually experience:
* **Time-to-First-Token (TTFT / Chunk 0 latency)** under high concurrency ($1$ to $500$ parallel connections).
* **Inter-Token Latency (ITL) & Streaming Jitter** ($\sigma$) to expose proxy chunk buffering and stream degradation.
* **HDR Latency Percentiles** ($p50$, $p90$, $p95$, $p99$, $p99.9$).
* **Standalone Self-Contained HTML Reports** (`--export-html report.html`) with embedded interactive SVG percentile curves.

---

## 🚀 Quickstart & Installation

### Install via Cargo (Crates.io)

```bash
cargo install sabrix-bench
```

### Or Install via Git

```bash
cargo install --git https://github.com/Pro-Kla/sabrix-bench
```

---

## 🛠️ CLI Usage

### 1. `sabrix-bench run` — Live HTTP/SSE Gateway Benchmark

Benchmark any external endpoint with concurrent workers and streaming SSE chunk evaluation:

```bash
# Benchmark local gateway with 50 parallel connections
sabrix-bench run --target http://localhost:8080/v1/chat/completions --concurrency 50 --requests 500

# Benchmark with embedded Enterprise RAG test corpus (50 prompts)
sabrix-bench run --target http://localhost:8080/v1/chat/completions --suite rag --concurrency 25 --requests 100

# Benchmark with OWASP LLM Top-10 & safety test suite
sabrix-bench run --target http://localhost:8080/v1/chat/completions --suite owasp --concurrency 20

# Export standalone zero-dependency dark-mode HTML report & JSON telemetry
sabrix-bench run \
  --target http://localhost:8080/v1/chat/completions \
  --concurrency 50 \
  --requests 1000 \
  --export-html benchmark_report.html \
  --export-json metrics.json

# Pass custom authentication or routing headers
sabrix-bench run \
  --target https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer sk-..." \
  --concurrency 10 \
  --requests 50
```

---

### 2. `sabrix-bench trace` — Real-Time MCP Tool-Call Security Inspector

Inspect JSON-RPC 2.0 requests (`tools/call`, `resources/read`) in sub-microsecond ($< 1\ \mu\text{s}$) latency and detect security violations (destructive shell commands, SQL mutations, credential leaks, path traversal):

```bash
# Run built-in demo scenarios
sabrix-bench trace --demo

# Inspect inline JSON payload
sabrix-bench trace -p '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "bash_exec",
    "arguments": { "cmd": "rm -rf /data/customers" }
  }
}'

# Pipe from stdin
cat mcp_payload.json | sabrix-bench trace
```

---

### 3. `sabrix-bench compare` — Multi-Turn Agent Latency Simulator

Calculate compounding latency penalties across multi-turn autonomous agent loops ($20$–$50$ turns) comparing In-Process safe-Rust evaluation vs. remote SaaS network roundtrips:

```bash
# Compare a 30-turn agent loop against a 120ms SaaS network baseline
sabrix-bench compare --turns 30

# Show architectural comparison matrix
sabrix-bench compare --matrix
```

---

## 📊 Public Test Corpora Included

`sabrix-bench` embeds standard test datasets directly inside the compiled binary:
* `--suite simple`: 10 low-overhead baseline health and connectivity prompts.
* `--suite rag`: 50 enterprise RAG prompts of varying lengths ($1\text{KB}$ to $32\text{KB}$) for saturation and throughput benchmarking.
* `--suite owasp`: 50 standardized security probes (prompt injection, sensitive path reads, secret leakage, SQL drops) and benign control prompts.
* `--payload <file.json>`: Custom user JSON request payloads.

---

## 🛡️ License

Dual licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
