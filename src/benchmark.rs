use crate::inspector::{McpInspector, RiskLevel};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnMetric {
    pub turn_index: usize,
    pub tool_name: String,
    pub payload_bytes: usize,
    pub serialization_us: f64,
    pub parsing_us: f64,
    pub inspection_us: f64,
    pub total_local_overhead_us: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDistribution {
    pub count: usize,
    pub min_us: f64,
    pub mean_us: f64,
    pub p50_us: f64,
    pub p95_us: f64,
    pub p99_us: f64,
    pub max_us: f64,
    pub std_dev_us: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub turns: usize,
    pub payload_scale: usize,
    pub quiet: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            turns: 20,
            payload_scale: 1,
            quiet: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub total_turns: usize,
    pub total_payload_bytes: usize,
    pub total_local_time_us: f64,
    pub distribution: LatencyDistribution,
    pub turns_detail: Vec<TurnMetric>,
    pub legacy_proxy_per_turn_ms: f64,
    pub saas_firewall_per_turn_ms: f64,
}

pub struct AgentBenchmark;

impl AgentBenchmark {
    fn generate_synthetic_turn_payload(turn: usize, scale: usize) -> (String, &'static str) {
        let (tool_name, args_val) = match turn % 7 {
            0 => (
                "read_file",
                serde_json::json!({"path": "src/core/agent.rs", "offset": 0, "limit": 100}),
            ),
            1 => (
                "search_codebase",
                serde_json::json!({"query": "fn handle_mcp_request", "file_types": ["rs", "toml"]}),
            ),
            2 => (
                "database_query",
                serde_json::json!({"sql": "SELECT id, name, email FROM users WHERE org_id = 'org_992' LIMIT 25"}),
            ),
            3 => (
                "fetch_weather",
                serde_json::json!({"city": "San Francisco", "units": "celsius"}),
            ),
            4 => (
                "vector_search",
                serde_json::json!({"collection": "documents", "vector": [0.12, 0.45, -0.23, 0.88, 0.04, -0.51], "k": 5}),
            ),
            5 => (
                "git_status",
                serde_json::json!({"repo": "sabrix-bench", "include_untracked": true}),
            ),
            _ => (
                "execute_command",
                serde_json::json!({"command": "cargo test --quiet"}),
            ),
        };

        let mut args = args_val;
        if scale > 1 {
            if let Some(map) = args.as_object_mut() {
                map.insert(
                    "context_padding".to_string(),
                    serde_json::Value::String("x".repeat(scale * 128)),
                );
            }
        }

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": turn + 1,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": args
            }
        });

        (payload.to_string(), tool_name)
    }

    pub fn run(config: &BenchmarkConfig) -> BenchmarkReport {
        let mut turn_metrics = Vec::with_capacity(config.turns);
        let mut total_payload_bytes = 0;

        let pb = if !config.quiet {
            let p = ProgressBar::new(config.turns as u64);
            p.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:.bold} [{bar:40.cyan/blue}] {pos}/{len} turns ({percent}%) - {msg}")
                    .expect("Valid template")
                    .progress_chars("█▓▒░"),
            );
            p.set_prefix("Benchmarking MCP Loop");
            Some(p)
        } else {
            None
        };

        for i in 0..config.turns {
            let (raw_json, tool_name) =
                Self::generate_synthetic_turn_payload(i, config.payload_scale);
            let bytes_len = raw_json.len();
            total_payload_bytes += bytes_len;

            // Measure serialize/clone overhead
            let t_ser_0 = Instant::now();
            let cloned_str = raw_json.clone();
            let serialization_us = (Instant::now() - t_ser_0).as_secs_f64() * 1_000_000.0;

            // Measure inspect and parse overhead
            let inspect_res = McpInspector::inspect_json_str(&cloned_str)
                .expect("Synthetic MCP payload should be valid JSON");

            let total_local_overhead_us = serialization_us + inspect_res.total_duration_us;

            turn_metrics.push(TurnMetric {
                turn_index: i + 1,
                tool_name: tool_name.to_string(),
                payload_bytes: bytes_len,
                serialization_us,
                parsing_us: inspect_res.parse_duration_us,
                inspection_us: inspect_res.inspection_duration_us,
                total_local_overhead_us,
                risk_level: inspect_res.max_risk_level,
            });

            if let Some(ref p) = pb {
                p.set_message(format!(
                    "Turn {}: {} ({:.1}µs)",
                    i + 1,
                    tool_name,
                    total_local_overhead_us
                ));
                p.inc(1);
            }
        }

        if let Some(ref p) = pb {
            p.finish_with_message("Done!");
        }

        let mut latencies: Vec<f64> = turn_metrics
            .iter()
            .map(|t| t.total_local_overhead_us)
            .collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = latencies.len();
        let min_us = latencies.first().copied().unwrap_or(0.0);
        let max_us = latencies.last().copied().unwrap_or(0.0);
        let sum: f64 = latencies.iter().sum();
        let mean_us = if count > 0 { sum / count as f64 } else { 0.0 };

        let p50_us = Self::percentile(&latencies, 50.0);
        let p95_us = Self::percentile(&latencies, 95.0);
        let p99_us = Self::percentile(&latencies, 99.0);

        let variance = if count > 1 {
            let sq_sum: f64 = latencies.iter().map(|&x| (x - mean_us).powi(2)).sum();
            sq_sum / (count - 1) as f64
        } else {
            0.0
        };
        let std_dev_us = variance.sqrt();

        let distribution = LatencyDistribution {
            count,
            min_us,
            mean_us,
            p50_us,
            p95_us,
            p99_us,
            max_us,
            std_dev_us,
        };

        BenchmarkReport {
            total_turns: config.turns,
            total_payload_bytes,
            total_local_time_us: sum,
            distribution,
            turns_detail: turn_metrics,
            legacy_proxy_per_turn_ms: 35.0,
            saas_firewall_per_turn_ms: 120.0,
        }
    }

    fn percentile(sorted: &[f64], pct: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let rank = (pct / 100.0) * (sorted.len() - 1) as f64;
        let lower = rank.floor() as usize;
        let upper = rank.ceil() as usize;
        if lower == upper {
            sorted[lower]
        } else {
            let weight = rank - lower as f64;
            sorted[lower] * (1.0 - weight) + sorted[upper] * weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_run() {
        let config = BenchmarkConfig {
            turns: 15,
            payload_scale: 1,
            quiet: true,
        };

        let report = AgentBenchmark::run(&config);
        assert_eq!(report.total_turns, 15);
        assert_eq!(report.turns_detail.len(), 15);
        assert!(report.distribution.mean_us > 0.0);
        assert!(report.distribution.p50_us <= report.distribution.p95_us);
        assert!(report.distribution.p95_us <= report.distribution.p99_us);
        assert!(report.distribution.p99_us <= report.distribution.max_us);
    }
}
