use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

mod benchmark;
mod inspector;
mod reporter;

use benchmark::{AgentBenchmark, BenchmarkConfig};
use inspector::McpInspector;
use reporter::Reporter;

#[derive(Parser, Debug)]
#[command(
    name = "sabrix-bench",
    author = "Chandradeep Neel <chandradeep@sabrix.ai>",
    version = "0.1.1",
    about = "⚡ Ultra-fast developer CLI & benchmark tool for Model Context Protocol (MCP) inspection and agent latency profiling.",
    long_about = "sabrix-bench is a lightweight, zero-bloat developer utility for inspecting MCP JSON-RPC messages, flagging dangerous tool calls, and benchmarking per-turn latency overhead across agent loops."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

    /// Compare architectural latency, privacy posture, and compounded costs
    Compare,
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
                // Read from stdin
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

        Commands::Compare => {
            Reporter::render_comparison_matrix();
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
