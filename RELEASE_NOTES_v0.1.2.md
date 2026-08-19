# Release v0.1.2: Hardened MCP Adversarial Security Engine ⚡

`sabrix-bench` v0.1.2 is officially published on [Crates.io](https://crates.io/crates/sabrix-bench). This release hardens the Model Context Protocol (MCP) inspection engine against real-world obfuscation, flag transpositions, intermediate URLs in shell pipes, modern credential formats, and path traversals while maintaining sub-microsecond (`< 2 µs`) execution in pure safe Rust.

---

## 🛡️ Key Hardening Updates & Highlights

- **`MCP-SEC-001` (Destructive Filesystem Hardening):**
  - Added detection for flag transpositions and separations: `rm -fr`, `rm -f -r`, `rm --recursive --force`.
  - Added signatures for alternative destructive file wipers: `find / -delete`, `shred -u`.

- **`MCP-SEC-002` (RCE & Piped Shell Hardening):**
  - Added pipeline detection for intermediate flags and URLs (e.g. `curl -sSL https://... | bash`, `wget -qO- ... | sh`).
  - Added support for alternative shell targets: `| zsh`, `| sudo sh`, `| sudo bash`, `| python`, `| perl`.

- **`MCP-SEC-003` (Dangerous SQL Queries):**
  - Added PostgreSQL/MySQL shorthand table wipe detection: `TRUNCATE customers;` (with or without the optional `TABLE` keyword).
  - Added tautological bypass detection: `WHERE 'a'='a'`, `OR 1=1`.
  - Added SQL block comment stripper (`/* ... */`).

- **`MCP-SEC-004` to `007` (Credentials & Secret Leakage):**
  - Added GitHub Fine-Grained Personal Access Tokens (`github_pat_...`).
  - Added AWS Temporary Session Tokens (`ASIA...`).
  - Added Google Cloud / Gemini API Keys (`AIzaSy...`).

- **`MCP-SEC-008` (Sensitive Local Paths):**
  - Added macOS alias path normalization (`/private/etc/passwd` $\to$ `/etc/passwd`).
  - Added relative path matching (`etc/passwd`, `etc/shadow`).
  - Added Windows SAM security registry database (`system32/config/sam`).

- **`MCP-SEC-009` (Unconstrained Execution Primitives):**
  - Expanded alias array to catch: `shell_exec`, `run_command`, `terminal`, `exec`, `powershell`, `cmd`, `zsh`.

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

- **17/17 Unit Tests Passing:** Covering deterministic security rules, flag swaps, intermediate piped URLs, token formats, SQL shorthands, path normalization, NDJSON streaming, and boundary turns.
- **Zero-Warning Guarantee:** Passes `cargo fmt --check` and `cargo clippy -- -D warnings`.

---

## 🛡️ Community & Governance

- Dual-licensed under **Apache-2.0** or **MIT** at your option.
- Contributions welcome! See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines on adding new MCP inspection rules.
- Production In-VPC Gateway: [https://sabrix.ai](https://sabrix.ai)
