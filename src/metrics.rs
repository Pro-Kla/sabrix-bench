use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSample {
    pub chunk_index: usize,
    pub arrival_ms: f64,
    pub delta_ms: f64,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetric {
    pub id: usize,
    pub status_code: u16,
    pub success: bool,
    pub ttft_ms: f64,
    pub total_duration_ms: f64,
    pub chunk_count: usize,
    pub bytes_received: usize,
    pub inter_chunk_deltas_ms: Vec<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PercentileStats {
    pub count: usize,
    pub min: f64,
    pub mean: f64,
    pub median: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub p99_9: f64,
    pub max: f64,
    pub std_dev: f64,
}

impl PercentileStats {
    pub fn compute(mut values: Vec<f64>) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let count = values.len();
        let min = values[0];
        let max = values[count - 1];
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let percentile = |p: f64| -> f64 {
            if count == 1 {
                return values[0];
            }
            let idx = (p / 100.0 * (count - 1) as f64).round() as usize;
            values[idx.min(count - 1)]
        };

        Self {
            count,
            min,
            mean,
            median: percentile(50.0),
            p90: percentile(90.0),
            p95: percentile(95.0),
            p99: percentile(99.0),
            p99_9: percentile(99.9),
            max,
            std_dev,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub target_url: String,
    pub suite_name: String,
    pub concurrency: usize,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub status_2xx: usize,
    pub status_4xx: usize,
    pub status_5xx: usize,
    pub total_duration_secs: f64,
    pub req_per_sec: f64,
    pub total_chunks: usize,
    pub chunks_per_sec: f64,
    pub total_bytes: usize,
    pub mb_per_sec: f64,
    
    // Core Distributions
    pub ttft_stats_ms: PercentileStats,
    pub jitter_stats_ms: PercentileStats,
    pub total_duration_stats_ms: PercentileStats,
    
    // Representative request stream samples for waterfall rendering
    pub sample_streams: Vec<Vec<ChunkSample>>,
}

impl BenchmarkReport {
    pub fn build(
        target_url: String,
        suite_name: String,
        concurrency: usize,
        total_test_duration_secs: f64,
        metrics: &[RequestMetric],
        sample_streams: Vec<Vec<ChunkSample>>,
    ) -> Self {
        let total_requests = metrics.len();
        let successful_requests = metrics.iter().filter(|m| m.success).count();
        let failed_requests = total_requests.saturating_sub(successful_requests);

        let status_2xx = metrics.iter().filter(|m| m.status_code >= 200 && m.status_code < 300).count();
        let status_4xx = metrics.iter().filter(|m| m.status_code >= 400 && m.status_code < 500).count();
        let status_5xx = metrics.iter().filter(|m| m.status_code >= 500 && m.status_code < 600).count();

        let total_chunks: usize = metrics.iter().map(|m| m.chunk_count).sum();
        let total_bytes: usize = metrics.iter().map(|m| m.bytes_received).sum();

        let req_per_sec = if total_test_duration_secs > 0.0 {
            total_requests as f64 / total_test_duration_secs
        } else {
            0.0
        };

        let chunks_per_sec = if total_test_duration_secs > 0.0 {
            total_chunks as f64 / total_test_duration_secs
        } else {
            0.0
        };

        let mb_per_sec = if total_test_duration_secs > 0.0 {
            (total_bytes as f64 / (1024.0 * 1024.0)) / total_test_duration_secs
        } else {
            0.0
        };

        // Extract TTFT values for successful requests
        let ttft_values: Vec<f64> = metrics
            .iter()
            .filter(|m| m.success && m.ttft_ms > 0.0)
            .map(|m| m.ttft_ms)
            .collect();
        let ttft_stats_ms = PercentileStats::compute(ttft_values);

        // Extract inter-chunk deltas (jitter)
        let mut all_deltas: Vec<f64> = Vec::new();
        for m in metrics {
            if m.success {
                all_deltas.extend(&m.inter_chunk_deltas_ms);
            }
        }
        let jitter_stats_ms = PercentileStats::compute(all_deltas);

        // Extract total durations
        let duration_values: Vec<f64> = metrics
            .iter()
            .filter(|m| m.success)
            .map(|m| m.total_duration_ms)
            .collect();
        let total_duration_stats_ms = PercentileStats::compute(duration_values);

        Self {
            target_url,
            suite_name,
            concurrency,
            total_requests,
            successful_requests,
            failed_requests,
            status_2xx,
            status_4xx,
            status_5xx,
            total_duration_secs: total_test_duration_secs,
            req_per_sec,
            total_chunks,
            chunks_per_sec,
            total_bytes,
            mb_per_sec,
            ttft_stats_ms,
            jitter_stats_ms,
            total_duration_stats_ms,
            sample_streams,
        }
    }
}
