use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

mod benchmark;
mod client;
mod inspector;
mod metrics;
mod reporter;
mod suites;

use benchmark::{AgentBenchmark, BenchmarkConfig};
use client::{BenchmarkClient, BenchmarkOptions};
use inspector::McpInspector;
use reporter::Reporter;
use suites::BenchmarkSuite;

#[derive(Parser, Debug)]
#[command(
    name = "sabrix-bench",
    author = "Chandradeep <chandradeep@sabrix.ai>",
    version,
    about = "⚡ Fast vendor-neutral benchmarking harness for AI proxies, firewalls, and LLM gateways.",
    long_about = "sabrix-bench is a lightweight, vendor-neutral benchmarking utility (the 'wrk / hyperfine for AI gateways') measuring pure client-visible TTFT, inter-token jitter, streaming tail latencies, and high-concurrency throughput across any proxy or LLM endpoint."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Benchmark an external AI proxy, LLM gateway, or target URL with live HTTP/SSE streaming
    Run {
        /// Target endpoint URL (e.g. http://localhost:8080/v1/chat/completions)
        #[arg(short, long, value_name = "URL")]
        target: String,

        /// Number of concurrent workers (1 to 500)
        #[arg(short, long, default_value_t = 10)]
        concurrency: usize,

        /// Total number of requests to dispatch
        #[arg(short, long, default_value_t = 100)]
        requests: usize,

        /// Built-in benchmark test suite: 'rag', 'owasp', or 'simple'
        #[arg(short, long, default_value = "simple")]
        suite: String,

        /// Custom JSON request payload file (overrides built-in suite)
        #[arg(short, long, value_name = "FILE")]
        payload: Option<PathBuf>,

        /// Enable chunk-by-chunk Server-Sent Events (SSE) streaming evaluation
        #[arg(long, default_value_t = true)]
        stream: bool,

        /// Custom HTTP request headers (e.g. -H "Authorization: Bearer sk-test")
        #[arg(short = 'H', long = "header", value_name = "KEY:VAL")]
        headers: Vec<String>,

        /// Path to export standalone, self-contained dark-mode HTML report
        #[arg(long, value_name = "HTML_PATH")]
        export_html: Option<PathBuf>,

        /// Path to export raw machine-readable JSON metrics
        #[arg(long, value_name = "JSON_PATH")]
        export_json: Option<PathBuf>,
    },

    /// Inspect MCP JSON-RPC 2.0 tool-calls and flag security risks in microseconds
    Trace {
        /// File containing MCP JSON-RPC message (reads from stdin if not specified)
        #[arg(short, long, value_name = "FILE")]
        input: Option<PathBuf>,

        /// Direct JSON-RPC payload string
        #[arg(short, long, value_name = "RAW_JSON")]
        payload: Option<String>,

        /// Run built-in demo inspecting typical safe & malicious tool payloads
        #[arg(long)]
        demo: bool,
    },

    /// Benchmark multi-turn agent loop serialization, inspection, and latency distribution
    Bench {
        /// Number of simulated agent loop turns
        #[arg(short, long, default_value_t = 20)]
        turns: usize,

        /// Payload size scale multiplier (1x = ~300 bytes, 10x = ~3.8 KB)
        #[arg(short, long, default_value_t = 1)]
        scale: usize,

        /// Suppress interactive progress bar
        #[arg(short, long)]
        quiet: bool,

        /// Output benchmark results as machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Compare dynamic multi-turn agent latency against remote SaaS AI firewalls
    Compare {
        /// Number of simulated agent loop turns
        #[arg(short, long, default_value_t = 30)]
        turns: usize,

        /// Baseline remote SaaS firewall latency per turn in milliseconds
        #[arg(long, default_value_t = 120.0)]
        saas_latency_ms: f64,

        /// Output comparison results as machine-readable JSON
        #[arg(long)]
        json: bool,

        /// Show static architectural capabilities matrix instead of live latency comparison
        #[arg(long)]
        matrix: bool,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("\n{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            target,
            concurrency,
            requests,
            suite,
            payload,
            stream,
            headers,
            export_html,
            export_json,
        } => {
            let payloads = if let Some(ref path) = payload {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read payload file: {}", path.display()))?;
                let val: serde_json::Value = serde_json::from_str(&content)
                    .context("Failed to parse custom payload as valid JSON")?;
                if let Some(arr) = val.as_array() {
                    arr.clone()
                } else {
                    vec![val]
                }
            } else {
                let suite_enum = BenchmarkSuite::from_str(&suite).unwrap_or_else(|| {
                    eprintln!(
                        "{} Unknown suite '{}'. Falling back to 'simple'. (Available: 'rag', 'owasp', 'simple')",
                        "Warning:".yellow().bold(),
                        suite
                    );
                    BenchmarkSuite::Simple
                });
                suite_enum.load_payloads()?
            };

            let suite_display_name = if payload.is_some() {
                "Custom User Payload File".to_string()
            } else {
                BenchmarkSuite::from_str(&suite)
                    .map(|s| s.name().to_string())
                    .unwrap_or_else(|| "Simple Baseline Suite".to_string())
            };

            println!();
            println!(
                "{} Dispatching {} requests (concurrency: {}) against {} ...",
                "⚡ [SABRIX-BENCH]".bright_cyan().bold(),
                requests,
                concurrency,
                target.bright_yellow().bold()
            );
            println!("   Corpus: {}", suite_display_name.white().bold());
            println!();

            let options = BenchmarkOptions {
                target_url: target.clone(),
                suite_name: suite_display_name,
                concurrency: concurrency.max(1),
                total_requests: requests.max(1),
                stream_mode: stream,
                custom_headers: headers,
                payloads,
            };

            let report = BenchmarkClient::execute(options).await?;

            Reporter::render_http_benchmark(&report);

            if let Some(html_path) = export_html {
                let path_str = html_path.to_string_lossy();
                Reporter::export_html_report(&report, &path_str)?;
                println!(
                    "{} Standalone HTML report exported to: {}",
                    "✓ [SAVED]".green().bold(),
                    path_str.bright_cyan().bold()
                );
                println!();
            }

            if let Some(json_path) = export_json {
                let path_str = json_path.to_string_lossy();
                let json_str = serde_json::to_string_pretty(&report)?;
                std::fs::write(&json_path, json_str)
                    .with_context(|| format!("Failed to write JSON report to {}", path_str))?;
                println!(
                    "{} Machine-readable JSON metrics exported to: {}",
                    "✓ [SAVED]".green().bold(),
                    path_str.bright_cyan().bold()
                );
                println!();
            }
        }

        Commands::Trace {
            input,
            payload,
            demo,
        } => {
            if demo {
                run_demo_traces()?;
                return Ok(());
            }

            let raw_json = if let Some(p) = payload {
                p
            } else if let Some(path) = input {
                std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read input file: {}", path.display()))?
            } else {
                let mut buffer = String::new();
                let mut stdin = io::stdin();

                if stdin.is_terminal() {
                    eprintln!(
                        "{} No input provided.\nPass a JSON payload via --payload, --input <file>, pipe from stdin, or run with --demo.",
                        "Notice:".yellow().bold()
                    );
                    std::process::exit(1);
                }

                stdin
                    .read_to_string(&mut buffer)
                    .context("Failed to read from standard input stream")?;

                if buffer.trim().is_empty() {
                    eprintln!("{} Stdin stream was empty.", "Notice:".yellow().bold());
                    std::process::exit(1);
                }
                buffer
            };

            let inspections = McpInspector::inspect_payload(&raw_json)
                .with_context(|| "Failed to process MCP JSON-RPC payload")?;

            for inspection in &inspections {
                Reporter::render_inspection(inspection);
            }
        }

        Commands::Bench {
            turns,
            scale,
            quiet,
            json,
        } => {
            let config = BenchmarkConfig {
                turns,
                payload_scale: scale,
                quiet: quiet || json,
            };

            let report = AgentBenchmark::run(&config);

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                Reporter::render_benchmark(&report);
            }
        }

        Commands::Compare {
            turns,
            saas_latency_ms,
            json,
            matrix,
        } => {
            if matrix {
                Reporter::render_comparison_matrix();
            } else {
                let report = AgentBenchmark::run_comparison(turns, saas_latency_ms);
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    Reporter::render_comparison(&report);
                }
            }
        }
    }

    Ok(())
}

fn run_demo_traces() -> Result<()> {
    println!("Running Sabrix MCP Inspector Demo...\n");

    let demo_payloads = [
        (
            "1. Safe Tool Call (Weather Forecast)",
            r#"{
                "jsonrpc": "2.0",
                "id": "call-1",
                "method": "tools/call",
                "params": {
                    "name": "get_weather",
                    "arguments": {
                        "city": "San Francisco",
                        "units": "celsius"
                    }
                }
            }"#,
        ),
        (
            "2. Destructive Shell Mutation (rm -rf)",
            r#"{
                "jsonrpc": "2.0",
                "id": "call-2",
                "method": "tools/call",
                "params": {
                    "name": "execute_command",
                    "arguments": {
                        "command": "rm -rf /var/data/production && echo done"
                    }
                }
            }"#,
        ),
        (
            "3. SQL Injection / Schema Deletion (DROP TABLE)",
            r#"{
                "jsonrpc": "2.0",
                "id": "call-3",
                "method": "tools/call",
                "params": {
                    "name": "database_query",
                    "arguments": {
                        "sql": "SELECT * FROM orders; DROP TABLE customers; --"
                    }
                }
            }"#,
        ),
        (
            "4. API Key & Credential Leakage in Payload",
            r#"{
                "jsonrpc": "2.0",
                "id": "call-4",
                "method": "tools/call",
                "params": {
                    "name": "http_request",
                    "arguments": {
                        "url": "https://api.openai.com/v1/chat/completions",
                        "auth_token": "sk-proj-9999988888777776666655555"
                    }
                }
            }"#,
        ),
    ];

    for (title, payload) in demo_payloads {
        println!(">>> Scenario: {}", title);
        let inspection = McpInspector::inspect_json_str(payload)?;
        Reporter::render_inspection(&inspection);
        println!();
    }

    Ok(())
}
