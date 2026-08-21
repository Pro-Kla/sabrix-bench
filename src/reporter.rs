use crate::benchmark::{BenchmarkReport as LocalBenchmarkReport, ComparisonReport};
use crate::inspector::{InspectionResult, RiskLevel};
use crate::metrics::BenchmarkReport as HttpBenchmarkReport;
use anyhow::{Context, Result};
use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use std::fs::File;
use std::io::Write;

pub struct Reporter;

impl Reporter {
    pub const CTA_FOOTER: &'static str =
        "Deploying AI agents & LLM gateways to production? Enforce zero-egress In-VPC security in < 2µs -> https://sabrix.ai";

    /// Formats and renders a single MCP JSON-RPC inspection result
    pub fn render_inspection(result: &InspectionResult) {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║                      SABRIX MCP TOOL-CALL INSPECTION                          ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();

        let mut meta_table = Table::new();
        meta_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        meta_table.set_header(vec![
            Cell::new("Property").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        meta_table.add_row(vec![
            Cell::new("Message Type"),
            Cell::new(if result.is_request {
                "JSON-RPC 2.0 Request"
            } else {
                "JSON-RPC 2.0 Response"
            }),
        ]);

        meta_table.add_row(vec![
            Cell::new("MCP Method"),
            Cell::new(&result.method).fg(Color::Cyan),
        ]);

        if let Some(ref tool) = result.tool_name {
            meta_table.add_row(vec![
                Cell::new("Target Tool / Resource"),
                Cell::new(tool)
                    .fg(Color::Yellow)
                    .add_attribute(Attribute::Bold),
            ]);
        }

        meta_table.add_row(vec![
            Cell::new("Payload Size"),
            Cell::new(format!("{} bytes", result.payload_bytes)),
        ]);

        let risk_cell = match result.max_risk_level {
            RiskLevel::Safe => Cell::new("PASSED (SAFE)")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            RiskLevel::Low => Cell::new("LOW RISK")
                .fg(Color::Blue)
                .add_attribute(Attribute::Bold),
            RiskLevel::Medium => Cell::new("MEDIUM RISK")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            RiskLevel::High => Cell::new("HIGH RISK")
                .fg(Color::DarkYellow)
                .add_attribute(Attribute::Bold),
            RiskLevel::Critical => Cell::new("CRITICAL THREAT")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
        };

        meta_table.add_row(vec![Cell::new("Security Status"), risk_cell]);

        meta_table.add_row(vec![
            Cell::new("Parsing Overhead"),
            Cell::new(format!("{:.2} µs", result.parse_duration_us)).fg(Color::Cyan),
        ]);

        meta_table.add_row(vec![
            Cell::new("Inspection Engine"),
            Cell::new(format!("{:.2} µs", result.inspection_duration_us)).fg(Color::Cyan),
        ]);

        meta_table.add_row(vec![
            Cell::new("Total Latency"),
            Cell::new(format!("{:.2} µs", result.total_duration_us))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);

        println!("{}", meta_table);
        println!();
    }

    /// Formats and renders the multi-turn agent latency benchmark report
    pub fn render_benchmark(report: &LocalBenchmarkReport) {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║             SABRIX IN-PROCESS AGENT OVERHEAD BENCHMARK                        ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Total Agent Turns"),
            Cell::new(format!("{}", report.total_turns)),
        ]);
        table.add_row(vec![
            Cell::new("Total Payload Inspected"),
            Cell::new(format!(
                "{} bytes ({:.2} KB)",
                report.total_payload_bytes,
                report.total_payload_bytes as f64 / 1024.0
            )),
        ]);
        table.add_row(vec![
            Cell::new("Mean Local Latency / Turn"),
            Cell::new(format!("{:.3} µs", report.distribution.mean_us))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);
        table.add_row(vec![
            Cell::new("Median (p50) Overhead"),
            Cell::new(format!("{:.3} µs", report.distribution.p50_us)).fg(Color::Cyan),
        ]);
        table.add_row(vec![
            Cell::new("95th Percentile (p95)"),
            Cell::new(format!("{:.3} µs", report.distribution.p95_us)).fg(Color::Yellow),
        ]);
        table.add_row(vec![
            Cell::new("99th Percentile (p99)"),
            Cell::new(format!("{:.3} µs", report.distribution.p99_us)).fg(Color::Red),
        ]);

        println!("{}", table);
        println!();
        Self::print_footer();
    }

    /// Formats and renders comparison between In-Process Sabrix and Remote SaaS Gateways
    pub fn render_comparison(report: &ComparisonReport) {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║        SABRIX MULTI-TURN AGENT LATENCY: IN-PROCESS VS. REMOTE SAAS            ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();

        if report.total_turns == 0 {
            println!("{}", "Zero turns benchmarked.".yellow());
            Self::print_footer();
            return;
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            Cell::new("Turn #").add_attribute(Attribute::Bold),
            Cell::new("Tool / Action").add_attribute(Attribute::Bold),
            Cell::new("In-Process Sabrix (µs)").add_attribute(Attribute::Bold),
            Cell::new("Remote SaaS Firewall (ms)").add_attribute(Attribute::Bold),
            Cell::new("Time Saved per Turn").add_attribute(Attribute::Bold),
        ]);

        for turn in &report.turns {
            let in_proc_str = format!("{:.2} µs", turn.in_process_us);
            let saas_str = format!("{:.1} ms", turn.saas_ms);
            let saved_str = format!("+{:.3} ms", turn.time_saved_ms);

            table.add_row(vec![
                Cell::new(format!("Turn {}", turn.turn_index)),
                Cell::new(&turn.tool_name).fg(Color::Yellow),
                Cell::new(in_proc_str)
                    .fg(Color::Green)
                    .add_attribute(Attribute::Bold),
                Cell::new(saas_str).fg(Color::Red),
                Cell::new(saved_str)
                    .fg(Color::Cyan)
                    .add_attribute(Attribute::Bold),
            ]);
        }

        println!("{}", table);
        println!();

        println!(
            "{}",
            format!(
                "📊 EXECUTIVE SUMMARY ({}-Turn Autonomous Agent Loop):",
                report.total_turns
            )
            .bold()
            .underline()
        );

        let in_proc_sec = report.total_in_process_ms / 1000.0;
        let saas_sec = report.total_saas_ms / 1000.0;
        let saved_sec = report.total_time_saved_ms / 1000.0;
        let egress_kb = report.total_egress_bytes_saved as f64 / 1024.0;

        println!(
            "  • {} {:.3} ms ({:.6} s)",
            "Total In-Process Governance Time:".bold(),
            report.total_in_process_ms,
            in_proc_sec
        );
        println!(
            "  • {} {:.1} ms ({:.2} s)",
            "Total Remote SaaS Firewall Latency:".bold(),
            report.total_saas_ms,
            saas_sec
        );
        println!(
            "  • {} {} ({} faster)",
            "Net Wall-Clock Latency Saved:".bold(),
            format!(
                "+{:.3} ms (+{:.3} s)",
                report.total_time_saved_ms, saved_sec
            )
            .green()
            .bold(),
            format!("{:.0}x", report.speedup_factor).cyan().bold()
        );
        println!(
            "  • {} {} (100% In-VPC / Zero Egress)",
            "Payload Data Egress Eliminated:".bold(),
            format!("{:.2} KB", egress_kb).yellow().bold()
        );
        println!();

        Self::print_footer();
    }

    /// Formats and renders the architecture comparison matrix
    pub fn render_comparison_matrix() {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║       AGENT SECURITY & RUNTIME ARCHITECTURAL COMPARISON MATRIX                ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec![
            Cell::new("Dimension / Capability").add_attribute(Attribute::Bold),
            Cell::new("Sabrix In-VPC Gateway")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("Legacy Python/Node Proxy")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("SaaS AI Firewalls")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Per-Turn Inspection Latency"),
            Cell::new("< 2 µs (Zero Overhead)")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("30 - 50 ms (Process/GC tax)").fg(Color::Yellow),
            Cell::new("100 - 250 ms (WAN + TLS)").fg(Color::Red),
        ]);

        table.add_row(vec![
            Cell::new("Compounded 30-Turn Loop Delay"),
            Cell::new("< 0.06 ms (Imperceptible)")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("+1.05 seconds").fg(Color::Yellow),
            Cell::new("+3.60 seconds (Laggy UX)").fg(Color::Red),
        ]);

        table.add_row(vec![
            Cell::new("Data Privacy & Egress"),
            Cell::new("100% In-VPC / Zero Egress")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("Local cluster (No external egress)"),
            Cell::new("Full payload egress to 3rd party").fg(Color::Red),
        ]);

        table.add_row(vec![
            Cell::new("MCP Protocol Native Support"),
            Cell::new("Native JSON-RPC 2.0 parser").fg(Color::Green),
            Cell::new("Ad-hoc JSON middleware"),
            Cell::new("Generic HTTP proxy"),
        ]);

        table.add_row(vec![
            Cell::new("Memory Footprint"),
            Cell::new("< 15 MB resident memory").fg(Color::Green),
            Cell::new("150 MB - 400 MB (Python/Node runtime)").fg(Color::Yellow),
            Cell::new("N/A (Managed SaaS)").fg(Color::Cyan),
        ]);

        table.add_row(vec![
            Cell::new("Threat Detection Mode"),
            Cell::new("Deterministic pre-execution (< 2µs)")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("Async middleware / Regex"),
            Cell::new("External cloud async LLM guard").fg(Color::Red),
        ]);

        table.add_row(vec![
            Cell::new("Cost Scaling"),
            Cell::new("Flat infrastructure / Zero egress cost").fg(Color::Green),
            Cell::new("Compute cluster scaling cost"),
            Cell::new("Per-token / Per-call billing markup").fg(Color::Red),
        ]);

        println!("{}", table);
        println!();

        Self::print_footer();
    }

    /// Renders the black-box HTTP/SSE benchmark terminal report
    pub fn render_http_benchmark(report: &HttpBenchmarkReport) {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║             ⚡ SABRIX AI PROXY & LLM GATEWAY BENCHMARK REPORT                 ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();

        let mut meta_table = Table::new();
        meta_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        meta_table.set_header(vec![
            Cell::new("Configuration").add_attribute(Attribute::Bold),
            Cell::new("Setting / Value").add_attribute(Attribute::Bold),
        ]);

        meta_table.add_row(vec![
            Cell::new("Target Endpoint"),
            Cell::new(&report.target_url).fg(Color::Cyan).add_attribute(Attribute::Bold),
        ]);
        meta_table.add_row(vec![
            Cell::new("Test Corpus"),
            Cell::new(&report.suite_name).fg(Color::Yellow),
        ]);
        meta_table.add_row(vec![
            Cell::new("Concurrency"),
            Cell::new(format!("{} parallel workers", report.concurrency)),
        ]);
        meta_table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(format!("{}", report.total_requests)),
        ]);
        meta_table.add_row(vec![
            Cell::new("Successful / Failed"),
            Cell::new(format!(
                "{} ok / {} failed (HTTP 2xx: {}, 4xx: {}, 5xx: {})",
                report.successful_requests, report.failed_requests, report.status_2xx, report.status_4xx, report.status_5xx
            ))
            .fg(if report.failed_requests == 0 { Color::Green } else { Color::Red }),
        ]);
        meta_table.add_row(vec![
            Cell::new("Throughput (req/s)"),
            Cell::new(format!("{:.2} req/sec", report.req_per_sec))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);
        meta_table.add_row(vec![
            Cell::new("Streaming Chunks / sec"),
            Cell::new(format!("{:.2} chunks/sec (Total: {})", report.chunks_per_sec, report.total_chunks))
                .fg(Color::Cyan),
        ]);
        meta_table.add_row(vec![
            Cell::new("Bandwidth"),
            Cell::new(format!("{:.2} MB/sec ({:.2} KB total)", report.mb_per_sec, report.total_bytes as f64 / 1024.0)),
        ]);

        println!("{}", meta_table);
        println!();

        let mut perc_table = Table::new();
        perc_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        perc_table.set_header(vec![
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new("Min").add_attribute(Attribute::Bold),
            Cell::new("Mean").add_attribute(Attribute::Bold),
            Cell::new("p50 (Med)").add_attribute(Attribute::Bold),
            Cell::new("p90").add_attribute(Attribute::Bold),
            Cell::new("p95").add_attribute(Attribute::Bold),
            Cell::new("p99").add_attribute(Attribute::Bold),
            Cell::new("p99.9").add_attribute(Attribute::Bold),
            Cell::new("Max").add_attribute(Attribute::Bold),
            Cell::new("Jitter (σ)").add_attribute(Attribute::Bold),
        ]);

        let ttft = &report.ttft_stats_ms;
        perc_table.add_row(vec![
            Cell::new("Time-to-First-Token (TTFT)").fg(Color::Cyan).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}ms", ttft.min)),
            Cell::new(format!("{:.2}ms", ttft.mean)),
            Cell::new(format!("{:.2}ms", ttft.median)).fg(Color::Green).add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}ms", ttft.p90)),
            Cell::new(format!("{:.2}ms", ttft.p95)),
            Cell::new(format!("{:.2}ms", ttft.p99)).fg(Color::Yellow),
            Cell::new(format!("{:.2}ms", ttft.p99_9)).fg(Color::Red),
            Cell::new(format!("{:.2}ms", ttft.max)),
            Cell::new(format!("±{:.2}ms", ttft.std_dev)),
        ]);

        let jitter = &report.jitter_stats_ms;
        if jitter.count > 0 {
            perc_table.add_row(vec![
                Cell::new("Inter-Chunk Delta (ITL)").fg(Color::Yellow),
                Cell::new(format!("{:.2}ms", jitter.min)),
                Cell::new(format!("{:.2}ms", jitter.mean)),
                Cell::new(format!("{:.2}ms", jitter.median)),
                Cell::new(format!("{:.2}ms", jitter.p90)),
                Cell::new(format!("{:.2}ms", jitter.p95)),
                Cell::new(format!("{:.2}ms", jitter.p99)),
                Cell::new(format!("{:.2}ms", jitter.p99_9)),
                Cell::new(format!("{:.2}ms", jitter.max)),
                Cell::new(format!("±{:.2}ms", jitter.std_dev)).fg(Color::Magenta),
            ]);
        }

        let tot = &report.total_duration_stats_ms;
        perc_table.add_row(vec![
            Cell::new("Total Stream Duration").fg(Color::White),
            Cell::new(format!("{:.2}ms", tot.min)),
            Cell::new(format!("{:.2}ms", tot.mean)),
            Cell::new(format!("{:.2}ms", tot.median)),
            Cell::new(format!("{:.2}ms", tot.p90)),
            Cell::new(format!("{:.2}ms", tot.p95)),
            Cell::new(format!("{:.2}ms", tot.p99)),
            Cell::new(format!("{:.2}ms", tot.p99_9)),
            Cell::new(format!("{:.2}ms", tot.max)),
            Cell::new(format!("±{:.2}ms", tot.std_dev)),
        ]);

        println!("{}", perc_table);
        println!();
        Self::print_footer();
    }

    /// Exports a self-contained, standalone, zero-dependency dark-mode HTML report
    pub fn export_html_report(report: &HttpBenchmarkReport, file_path: &str) -> Result<()> {
        let ttft = &report.ttft_stats_ms;
        let jitter = &report.jitter_stats_ms;
        let tot = &report.total_duration_stats_ms;

        let max_val = ttft.max.max(1.0);
        let svg_points = format!(
            "0,{:.1} 100,{:.1} 200,{:.1} 300,{:.1} 400,{:.1} 500,{:.1}",
            180.0 - (ttft.min / max_val * 140.0),
            180.0 - (ttft.median / max_val * 140.0),
            180.0 - (ttft.p90 / max_val * 140.0),
            180.0 - (ttft.p95 / max_val * 140.0),
            180.0 - (ttft.p99 / max_val * 140.0),
            180.0 - (ttft.p99_9 / max_val * 140.0),
        );

        let html_content = format!(
            r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Sabrix Benchmark Report — {target_url}</title>
  <style>
    :root {{
      --bg: #070a12;
      --card-bg: #0d1322;
      --border: #1e293b;
      --cyan: #06b6d4;
      --indigo: #6366f1;
      --green: #10b981;
      --yellow: #f59e0b;
      --red: #ef4444;
      --text: #f8fafc;
      --text-muted: #94a3b8;
      --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
      --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background: var(--bg);
      color: var(--text);
      font-family: var(--font-sans);
      line-height: 1.5;
      padding: 2rem 1.5rem;
    }}
    .container {{ max-width: 1100px; margin: 0 auto; }}
    .header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 1rem;
      border-bottom: 1px solid var(--border);
      padding-bottom: 1.5rem;
      margin-bottom: 2rem;
    }}
    .brand {{ display: flex; align-items: center; gap: 10px; font-weight: 800; font-size: 1.25rem; }}
    .tag {{
      font-family: var(--font-mono);
      font-size: 0.75rem;
      padding: 3px 8px;
      border-radius: 4px;
      background: rgba(6, 182, 212, 0.12);
      border: 1px solid rgba(6, 182, 212, 0.3);
      color: var(--cyan);
      font-weight: 600;
    }}
    .grid-4 {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; margin-bottom: 2rem; }}
    .kpi-card {{
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.25rem;
      display: flex;
      flex-direction: column;
      gap: 4px;
      position: relative;
      overflow: hidden;
    }}
    .kpi-card::before {{
      content: ""; position: absolute; top: 0; left: 0; right: 0; height: 2px;
      background: linear-gradient(90deg, var(--cyan), var(--indigo));
    }}
    .kpi-val {{ font-family: var(--font-mono); font-size: 1.85rem; font-weight: 800; color: #ffffff; }}
    .kpi-lbl {{ font-size: 0.8rem; font-weight: 700; text-transform: uppercase; color: var(--text-muted); }}
    .kpi-sub {{ font-size: 0.75rem; color: var(--cyan); font-family: var(--font-mono); }}

    .card {{
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1.5rem;
      margin-bottom: 2rem;
    }}
    .card-title {{ font-size: 1.1rem; font-weight: 700; margin-bottom: 1rem; display: flex; justify-content: space-between; align-items: center; }}

    table {{ width: 100%; border-collapse: collapse; font-family: var(--font-mono); font-size: 0.85rem; }}
    th, td {{ padding: 10px 14px; text-align: left; border-bottom: 1px solid var(--border); }}
    th {{ color: var(--text-muted); font-weight: 600; text-transform: uppercase; font-size: 0.75rem; }}
    tr:last-child td {{ border-bottom: none; }}

    .chart-container {{
      background: #050810;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 1.5rem;
      margin-top: 1rem;
    }}
    .footer {{
      text-align: center;
      border-top: 1px solid var(--border);
      padding-top: 1.5rem;
      font-size: 0.8rem;
      color: var(--text-muted);
      font-family: var(--font-mono);
    }}
  </style>
</head>
<body>
  <div class="container">
    
    <header class="header">
      <div class="brand">
        <span style="color:var(--cyan)">⚡</span>
        <span>SABRIX<span style="color:var(--cyan)">.AI</span></span>
        <span class="tag">AI Gateway Benchmark v0.2.0</span>
      </div>
      <div style="font-family:var(--font-mono);font-size:0.8rem;color:var(--text-muted)">
        Target: <code style="color:var(--cyan);font-weight:700">{target_url}</code>
      </div>
    </header>

    <!-- 4 High-Impact KPI Cards -->
    <div class="grid-4">
      <div class="kpi-card">
        <div class="kpi-lbl">Throughput</div>
        <div class="kpi-val" style="color:var(--green)">{req_per_sec:.1} <span style="font-size:1rem">req/s</span></div>
        <div class="kpi-sub">{chunks_per_sec:.1} chunks/sec</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-lbl">TTFT (p50 Median)</div>
        <div class="kpi-val" style="color:var(--cyan)">{ttft_p50:.2} <span style="font-size:1rem">ms</span></div>
        <div class="kpi-sub">Min: {ttft_min:.2}ms</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-lbl">TTFT Tail (p99)</div>
        <div class="kpi-val" style="color:var(--yellow)">{ttft_p99:.2} <span style="font-size:1rem">ms</span></div>
        <div class="kpi-sub">p99.9: {ttft_p99_9:.2}ms</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-lbl">Streaming Jitter (σ)</div>
        <div class="kpi-val" style="color:var(--indigo)">±{jitter_std:.2} <span style="font-size:1rem">ms</span></div>
        <div class="kpi-sub">Mean ITL: {jitter_mean:.2}ms</div>
      </div>
    </div>

    <!-- Latency Percentile Curve (SVG Chart) -->
    <div class="card">
      <div class="card-title">
        <span>📈 Latency Percentile Distribution (p50 → p99.9)</span>
        <span class="tag">Empirical HDR Distribution</span>
      </div>
      <div class="chart-container">
        <svg viewBox="0 0 500 200" width="100%" height="220" style="overflow:visible">
          <line x1="0" y1="40" x2="500" y2="40" stroke="#1e293b" stroke-dasharray="4"/>
          <line x1="0" y1="90" x2="500" y2="90" stroke="#1e293b" stroke-dasharray="4"/>
          <line x1="0" y1="140" x2="500" y2="140" stroke="#1e293b" stroke-dasharray="4"/>
          <line x1="0" y1="190" x2="500" y2="190" stroke="#334155"/>
          
          <polygon points="0,190 {svg_points} 500,190" fill="rgba(6, 182, 212, 0.12)"/>
          <polyline fill="none" stroke="#06b6d4" stroke-width="3" points="{svg_points}"/>

          <text x="0" y="210" fill="#94a3b8" font-size="11" font-family="monospace">Min</text>
          <text x="100" y="210" fill="#94a3b8" font-size="11" font-family="monospace">p50</text>
          <text x="200" y="210" fill="#94a3b8" font-size="11" font-family="monospace">p90</text>
          <text x="300" y="210" fill="#94a3b8" font-size="11" font-family="monospace">p95</text>
          <text x="400" y="210" fill="#94a3b8" font-size="11" font-family="monospace">p99</text>
          <text x="470" y="210" fill="#94a3b8" font-size="11" font-family="monospace">p99.9</text>
        </svg>
      </div>
    </div>

    <!-- Detailed Metrics Table -->
    <div class="card">
      <div class="card-title">
        <span>📊 Complete Percentile Latency Matrix</span>
      </div>
      <table>
        <thead>
          <tr>
            <th>Metric Layer</th>
            <th>Min</th>
            <th>Mean</th>
            <th>p50 (Median)</th>
            <th>p90</th>
            <th>p95</th>
            <th>p99</th>
            <th>p99.9</th>
            <th>Max</th>
            <th>Std Dev (σ)</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td style="color:var(--cyan);font-weight:700">Time-to-First-Token (TTFT)</td>
            <td>{ttft_min:.2}ms</td>
            <td>{ttft_mean:.2}ms</td>
            <td style="color:var(--green);font-weight:700">{ttft_p50:.2}ms</td>
            <td>{ttft_p90:.2}ms</td>
            <td>{ttft_p95:.2}ms</td>
            <td style="color:var(--yellow);font-weight:700">{ttft_p99:.2}ms</td>
            <td style="color:var(--red);font-weight:700">{ttft_p99_9:.2}ms</td>
            <td>{ttft_max:.2}ms</td>
            <td>±{ttft_std:.2}ms</td>
          </tr>
          <tr>
            <td style="color:var(--yellow)">Inter-Token Jitter (ITL)</td>
            <td>{jit_min:.2}ms</td>
            <td>{jit_mean:.2}ms</td>
            <td>{jit_p50:.2}ms</td>
            <td>{jit_p90:.2}ms</td>
            <td>{jit_p95:.2}ms</td>
            <td>{jit_p99:.2}ms</td>
            <td>{jit_p99_9:.2}ms</td>
            <td>{jit_max:.2}ms</td>
            <td style="color:var(--indigo);font-weight:700">±{jitter_std:.2}ms</td>
          </tr>
          <tr>
            <td style="color:#ffffff">Total Stream Duration</td>
            <td>{tot_min:.2}ms</td>
            <td>{tot_mean:.2}ms</td>
            <td>{tot_p50:.2}ms</td>
            <td>{tot_p90:.2}ms</td>
            <td>{tot_p95:.2}ms</td>
            <td>{tot_p99:.2}ms</td>
            <td>{tot_p99_9:.2}ms</td>
            <td>{tot_max:.2}ms</td>
            <td>±{tot_std:.2}ms</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Execution Environment Metadata -->
    <div class="card">
      <div class="card-title">
        <span>⚙️ Benchmark Execution Environment</span>
      </div>
      <table>
        <tbody>
          <tr>
            <td style="color:var(--text-muted)">Target URL</td>
            <td><code>{target_url}</code></td>
            <td style="color:var(--text-muted)">Benchmark Suite</td>
            <td><code>{suite_name}</code></td>
          </tr>
          <tr>
            <td style="color:var(--text-muted)">Concurrency Pool</td>
            <td><strong>{concurrency} workers</strong></td>
            <td style="color:var(--text-muted)">Total Requests</td>
            <td><strong>{total_requests} requests</strong></td>
          </tr>
          <tr>
            <td style="color:var(--text-muted)">HTTP Status Distribution</td>
            <td><span style="color:var(--green)">{status_2xx} (2xx OK)</span> · <span style="color:var(--yellow)">{status_4xx} (4xx)</span> · <span style="color:var(--red)">{status_5xx} (5xx)</span></td>
            <td style="color:var(--text-muted)">Total Stream Duration</td>
            <td><strong>{total_duration:.2} seconds</strong></td>
          </tr>
        </tbody>
      </table>
    </div>

    <footer class="footer">
      Generated by <strong>sabrix-bench v0.2.0</strong> · Zero-Egress In-VPC Autonomous Ingress Governance · <a href="https://sabrix.ai" target="_blank" style="color:var(--cyan);text-decoration:none">https://sabrix.ai</a>
    </footer>

  </div>
</body>
</html>"##,
            target_url = report.target_url,
            suite_name = report.suite_name,
            concurrency = report.concurrency,
            total_requests = report.total_requests,
            req_per_sec = report.req_per_sec,
            chunks_per_sec = report.chunks_per_sec,
            status_2xx = report.status_2xx,
            status_4xx = report.status_4xx,
            status_5xx = report.status_5xx,
            total_duration = report.total_duration_secs,
            ttft_min = ttft.min,
            ttft_mean = ttft.mean,
            ttft_p50 = ttft.median,
            ttft_p90 = ttft.p90,
            ttft_p95 = ttft.p95,
            ttft_p99 = ttft.p99,
            ttft_p99_9 = ttft.p99_9,
            ttft_max = ttft.max,
            ttft_std = ttft.std_dev,
            jit_min = jitter.min,
            jit_mean = jitter.mean,
            jit_p50 = jitter.median,
            jit_p90 = jitter.p90,
            jit_p95 = jitter.p95,
            jit_p99 = jitter.p99,
            jit_p99_9 = jitter.p99_9,
            jit_max = jitter.max,
            jitter_mean = jitter.mean,
            jitter_std = jitter.std_dev,
            tot_min = tot.min,
            tot_mean = tot.mean,
            tot_p50 = tot.median,
            tot_p90 = tot.p90,
            tot_p95 = tot.p95,
            tot_p99 = tot.p99,
            tot_p99_9 = tot.p99_9,
            tot_max = tot.max,
            tot_std = tot.std_dev,
            svg_points = svg_points,
        );

        let mut file = File::create(file_path)
            .with_context(|| format!("Failed to create HTML report file at {}", file_path))?;
        file.write_all(html_content.as_bytes())
            .with_context(|| format!("Failed to write HTML report content to {}", file_path))?;

        Ok(())
    }

    pub fn print_footer() {
        println!("{}", "-".repeat(80).dimmed());
        println!("{} {}", "🚀".bold(), Self::CTA_FOOTER.bright_cyan().bold());
        println!("{}", "-".repeat(80).dimmed());
        println!();
    }
}
