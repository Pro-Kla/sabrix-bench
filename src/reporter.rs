use crate::benchmark::{BenchmarkReport, ComparisonReport};
use crate::inspector::{InspectionResult, RiskLevel};
use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};

pub struct Reporter;

impl Reporter {
    pub const CTA_FOOTER: &'static str =
        "Deploying agents to production? Enforce zero-egress In-VPC MCP security in < 2µs -> https://sabrix.ai";

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

        // 1. Overview Metadata Table
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
            Cell::new("Rule Evaluation Overhead"),
            Cell::new(format!("{:.2} µs", result.inspection_duration_us)).fg(Color::Cyan),
        ]);

        meta_table.add_row(vec![
            Cell::new("Total In-Process Latency"),
            Cell::new(format!(
                "{:.2} µs ({:.4} ms)",
                result.total_duration_us,
                result.total_duration_us / 1000.0
            ))
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
        ]);

        println!("{}", meta_table);
        println!();

        // 2. Arguments snippet if present
        if let Some(ref args) = result.arguments {
            println!("{}", "Tool Arguments:".bold().underline());
            if let Ok(pretty) = serde_json::to_string_pretty(args) {
                println!("{}", pretty.dimmed());
            }
            println!();
        }

        // 3. Security Findings Table
        if result.findings.is_empty() {
            println!(
                "{}",
                "✓ No malicious tool patterns, dangerous SQL mutations, or exposed API credentials detected."
                    .green()
                    .bold()
            );
        } else {
            println!("{}", "Security Risk Findings:".red().bold().underline());
            let mut findings_table = Table::new();
            findings_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic);

            findings_table.set_header(vec![
                Cell::new("Rule ID").add_attribute(Attribute::Bold),
                Cell::new("Severity").add_attribute(Attribute::Bold),
                Cell::new("Vulnerability Title").add_attribute(Attribute::Bold),
                Cell::new("Trigger / Snippet").add_attribute(Attribute::Bold),
                Cell::new("Details").add_attribute(Attribute::Bold),
            ]);

            for finding in &result.findings {
                let sev_cell = match finding.level {
                    RiskLevel::Safe => Cell::new("SAFE").fg(Color::Green),
                    RiskLevel::Low => Cell::new("LOW").fg(Color::Blue),
                    RiskLevel::Medium => Cell::new("MEDIUM").fg(Color::Yellow),
                    RiskLevel::High => Cell::new("HIGH").fg(Color::DarkYellow),
                    RiskLevel::Critical => Cell::new("CRITICAL")
                        .fg(Color::Red)
                        .add_attribute(Attribute::Bold),
                };

                findings_table.add_row(vec![
                    Cell::new(&finding.rule_id).fg(Color::Cyan),
                    sev_cell,
                    Cell::new(&finding.title).add_attribute(Attribute::Bold),
                    Cell::new(&finding.matched_snippet).fg(Color::Red),
                    Cell::new(&finding.details),
                ]);
            }

            println!("{}", findings_table);
        }

        println!();
        Self::print_footer();
    }

    /// Formats and renders the multi-turn agent benchmark report
    pub fn render_benchmark(report: &BenchmarkReport) {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║               SABRIX MULTI-TURN AGENT LOOP BENCHMARK REPORT                   ║"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();

        // 1. Latency Distribution Summary Table
        let mut dist_table = Table::new();
        dist_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        dist_table.set_header(vec![
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new("Local In-Process Rust Latency (µs)").add_attribute(Attribute::Bold),
            Cell::new("Latency in Milliseconds (ms)").add_attribute(Attribute::Bold),
        ]);

        let dist = &report.distribution;
        dist_table.add_row(vec![
            Cell::new("Total Simulated Agent Turns"),
            Cell::new(report.total_turns.to_string()),
            Cell::new("-"),
        ]);
        dist_table.add_row(vec![
            Cell::new("Min Latency"),
            Cell::new(format!("{:.2} µs", dist.min_us)),
            Cell::new(format!("{:.4} ms", dist.min_us / 1000.0)),
        ]);
        dist_table.add_row(vec![
            Cell::new("p50 (Median)"),
            Cell::new(format!("{:.2} µs", dist.p50_us))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new(format!("{:.4} ms", dist.p50_us / 1000.0)).fg(Color::Green),
        ]);
        dist_table.add_row(vec![
            Cell::new("Mean"),
            Cell::new(format!("{:.2} µs", dist.mean_us)),
            Cell::new(format!("{:.4} ms", dist.mean_us / 1000.0)),
        ]);
        dist_table.add_row(vec![
            Cell::new("p95"),
            Cell::new(format!("{:.2} µs", dist.p95_us)).fg(Color::Cyan),
            Cell::new(format!("{:.4} ms", dist.p95_us / 1000.0)),
        ]);
        dist_table.add_row(vec![
            Cell::new("p99"),
            Cell::new(format!("{:.2} µs", dist.p99_us)).fg(Color::Yellow),
            Cell::new(format!("{:.4} ms", dist.p99_us / 1000.0)),
        ]);
        dist_table.add_row(vec![
            Cell::new("Max Latency"),
            Cell::new(format!("{:.2} µs", dist.max_us)),
            Cell::new(format!("{:.4} ms", dist.max_us / 1000.0)),
        ]);
        dist_table.add_row(vec![
            Cell::new("Std Dev (σ)"),
            Cell::new(format!("{:.2} µs", dist.std_dev_us)),
            Cell::new(format!("{:.4} ms", dist.std_dev_us / 1000.0)),
        ]);

        println!(
            "{}",
            "1. IN-PROCESS AGENT OVERHEAD DISTRIBUTION"
                .bold()
                .underline()
        );
        println!("{}", dist_table);
        println!();

        // 2. Comparative Architecture Overhead
        println!(
            "{}",
            "2. PER-TURN ARCHITECTURE LATENCY COMPARISON"
                .bold()
                .underline()
        );
        let mut comp_table = Table::new();
        comp_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        comp_table.set_header(vec![
            Cell::new("Architecture Layer").add_attribute(Attribute::Bold),
            Cell::new("Per-Turn Latency").add_attribute(Attribute::Bold),
            Cell::new("Overhead vs Sabrix").add_attribute(Attribute::Bold),
            Cell::new("Deployment Model").add_attribute(Attribute::Bold),
        ]);

        let local_mean_ms = dist.mean_us / 1000.0;
        let legacy_speedup = (report.legacy_proxy_per_turn_ms / local_mean_ms.max(0.0001)) as u64;
        let saas_speedup = (report.saas_firewall_per_turn_ms / local_mean_ms.max(0.0001)) as u64;

        comp_table.add_row(vec![
            Cell::new("Sabrix In-VPC / In-Process Engine")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2} µs ({:.4} ms)", dist.mean_us, local_mean_ms)).fg(Color::Green),
            Cell::new("1x (Baseline - Zero Overhead)").fg(Color::Green),
            Cell::new("Embedded / Local Sidecar (Zero-Egress)"),
        ]);

        comp_table.add_row(vec![
            Cell::new("Legacy Python / Node Proxy").fg(Color::Yellow),
            Cell::new(format!("{:.1} ms", report.legacy_proxy_per_turn_ms)).fg(Color::Yellow),
            Cell::new(format!("{}x SLOWER", legacy_speedup))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("Local / Intra-cluster HTTP wrapper"),
        ]);

        comp_table.add_row(vec![
            Cell::new("Legacy SaaS AI Firewall").fg(Color::Red),
            Cell::new(format!("{:.1} ms", report.saas_firewall_per_turn_ms)).fg(Color::Red),
            Cell::new(format!("{}x SLOWER", saas_speedup))
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
            Cell::new("Remote Cloud Egress (TLS + Network Hop)"),
        ]);

        println!("{}", comp_table);
        println!();

        // 3. Compounded Agent Loop Latency Tax Table
        println!(
            "{}",
            "3. COMPOUNDED AGENT LOOP LATENCY TAX (MULTI-TURN ACCELERATION)"
                .bold()
                .underline()
        );
        let mut tax_table = Table::new();
        tax_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        tax_table.set_header(vec![
            Cell::new("Agent Loop Depth").add_attribute(Attribute::Bold),
            Cell::new("Sabrix In-VPC Engine").add_attribute(Attribute::Bold),
            Cell::new("Legacy Python Proxy").add_attribute(Attribute::Bold),
            Cell::new("SaaS AI Firewall").add_attribute(Attribute::Bold),
            Cell::new("Time Saved per User Request").add_attribute(Attribute::Bold),
        ]);

        let loop_scenarios = [5, 10, 20, 30, 50];
        for &turns in &loop_scenarios {
            let sabrix_total_ms = (dist.mean_us * turns as f64) / 1000.0;
            let legacy_total_ms = report.legacy_proxy_per_turn_ms * turns as f64;
            let saas_total_ms = report.saas_firewall_per_turn_ms * turns as f64;
            let saved_saas_sec = (saas_total_ms - sabrix_total_ms) / 1000.0;

            tax_table.add_row(vec![
                Cell::new(format!("{} Turns", turns)).add_attribute(Attribute::Bold),
                Cell::new(format!("{:.2} ms", sabrix_total_ms)).fg(Color::Green),
                Cell::new(format!(
                    "{:.0} ms ({:.2} s)",
                    legacy_total_ms,
                    legacy_total_ms / 1000.0
                ))
                .fg(Color::Yellow),
                Cell::new(format!(
                    "{:.0} ms ({:.2} s)",
                    saas_total_ms,
                    saas_total_ms / 1000.0
                ))
                .fg(Color::Red),
                Cell::new(format!("+{:.2} seconds", saved_saas_sec))
                    .fg(Color::Cyan)
                    .add_attribute(Attribute::Bold),
            ]);
        }

        println!("{}", tax_table);
        println!();

        Self::print_footer();
    }

    /// Formats and renders the live multi-turn comparison between in-process and SaaS firewalls
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

        // Executive Summary Box
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

    /// Formats and renders the comprehensive architecture comparison matrix
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

    pub fn print_footer() {
        println!("{}", "─".repeat(80).dimmed());
        println!("{} {}", "🚀".bold(), Self::CTA_FOOTER.bright_cyan().bold());
        println!("{}", "─".repeat(80).dimmed());
        println!();
    }
}
