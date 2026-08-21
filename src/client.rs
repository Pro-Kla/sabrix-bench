use crate::metrics::{BenchmarkReport, ChunkSample, RequestMetric};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BenchmarkOptions {
    pub target_url: String,
    pub suite_name: String,
    pub concurrency: usize,
    pub total_requests: usize,
    pub stream_mode: bool,
    pub custom_headers: Vec<String>,
    pub payloads: Vec<Value>,
}

pub struct BenchmarkClient;

impl BenchmarkClient {
    pub async fn execute(options: BenchmarkOptions) -> Result<BenchmarkReport> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .pool_max_idle_per_host(options.concurrency + 10)
            .build()
            .context("Failed to initialize HTTP client")?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if options.stream_mode {
            default_headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
        } else {
            default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        }

        for h in &options.custom_headers {
            if let Some((k, v)) = h.split_once(':') {
                let name = HeaderName::from_str(k.trim()).context("Invalid header name")?;
                let val = HeaderValue::from_str(v.trim()).context("Invalid header value")?;
                default_headers.insert(name, val);
            }
        }

        let pb = ProgressBar::new(options.total_requests as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} reqs ({per_sec}) | ETA: {eta}")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("█▓▒░"),
        );

        let (tx, rx) = mpsc::channel::<(usize, Value)>(options.total_requests);
        let rx_arc = Arc::new(Mutex::new(rx));

        // Enqueue work
        let payloads_len = options.payloads.len().max(1);
        for id in 0..options.total_requests {
            let p = options.payloads[id % payloads_len].clone();
            tx.send((id, p)).await.ok();
        }
        drop(tx);

        let results_arc = Arc::new(Mutex::new(Vec::with_capacity(options.total_requests)));
        let sample_streams_arc: Arc<Mutex<Vec<Vec<ChunkSample>>>> = Arc::new(Mutex::new(Vec::new()));

        let start_time = Instant::now();
        let mut handles = Vec::with_capacity(options.concurrency);

        for _ in 0..options.concurrency {
            let client_clone = client.clone();
            let target_url_clone = options.target_url.clone();
            let headers_clone = default_headers.clone();
            let rx_worker = Arc::clone(&rx_arc);
            let results_worker = Arc::clone(&results_arc);
            let sample_streams_worker = Arc::clone(&sample_streams_arc);
            let pb_worker = pb.clone();
            let stream_mode = options.stream_mode;

            handles.push(tokio::spawn(async move {
                loop {
                    let task = {
                        let mut locked_rx = rx_worker.lock().await;
                        locked_rx.recv().await
                    };

                    let (req_id, payload) = match task {
                        Some(t) => t,
                        None => break, // Work queue drained
                    };

                    let req_res = Self::send_single_request(
                        &client_clone,
                        &target_url_clone,
                        &headers_clone,
                        payload,
                        req_id,
                        stream_mode,
                    )
                    .await;

                    {
                        let mut locked_res = results_worker.lock().await;
                        if req_id < 5 && req_res.1.is_some() {
                            let mut locked_samples = sample_streams_worker.lock().await;
                            if let Some(s) = req_res.1 {
                                locked_samples.push(s);
                            }
                        }
                        locked_res.push(req_res.0);
                    }

                    pb_worker.inc(1);
                }
            }));
        }

        for h in handles {
            h.await.ok();
        }

        let total_duration = start_time.elapsed().as_secs_f64();
        pb.finish_with_message("Benchmark Completed");

        let final_results = results_arc.lock().await.clone();
        let sample_streams = sample_streams_arc.lock().await.clone();

        Ok(BenchmarkReport::build(
            options.target_url,
            options.suite_name,
            options.concurrency,
            total_duration,
            &final_results,
            sample_streams,
        ))
    }

    async fn send_single_request(
        client: &reqwest::Client,
        target_url: &str,
        headers: &HeaderMap,
        payload: Value,
        req_id: usize,
        stream_mode: bool,
    ) -> (RequestMetric, Option<Vec<ChunkSample>>) {
        let t0 = Instant::now();

        let req_builder = client
            .post(target_url)
            .headers(headers.clone())
            .json(&payload);

        let response_res = req_builder.send().await;
        let response = match response_res {
            Ok(resp) => resp,
            Err(e) => {
                let dur_ms = t0.elapsed().as_secs_f64() * 1000.0;
                return (
                    RequestMetric {
                        id: req_id,
                        status_code: 0,
                        success: false,
                        ttft_ms: 0.0,
                        total_duration_ms: dur_ms,
                        chunk_count: 0,
                        bytes_received: 0,
                        inter_chunk_deltas_ms: vec![],
                        error: Some(e.to_string()),
                    },
                    None,
                );
            }
        };

        let status_code = response.status().as_u16();
        let success = response.status().is_success();

        let mut ttft_ms = 0.0;
        let mut total_bytes = 0;
        let mut chunk_count = 0;
        let mut inter_chunk_deltas = Vec::new();
        let mut chunk_samples = Vec::new();
        let mut prev_chunk_time = t0;

        if stream_mode && success {
            let mut stream = response.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let now = Instant::now();
                match chunk_result {
                    Ok(bytes) => {
                        let bytes_len = bytes.len();
                        total_bytes += bytes_len;
                        chunk_count += 1;

                        let delta_ms = (now - prev_chunk_time).as_secs_f64() * 1000.0;
                        if chunk_count == 1 {
                            ttft_ms = (now - t0).as_secs_f64() * 1000.0;
                        } else {
                            inter_chunk_deltas.push(delta_ms);
                        }

                        let is_done = bytes.windows(6).any(|w| w == b"[DONE]");

                        if chunk_samples.len() < 50 {
                            chunk_samples.push(ChunkSample {
                                chunk_index: chunk_count,
                                arrival_ms: (now - t0).as_secs_f64() * 1000.0,
                                delta_ms,
                                bytes: bytes_len,
                            });
                        }

                        prev_chunk_time = now;

                        if is_done {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        } else {
            let body_bytes = response.bytes().await.unwrap_or_default();
            total_bytes = body_bytes.len();
            chunk_count = 1;
            ttft_ms = t0.elapsed().as_secs_f64() * 1000.0;
        }

        let total_duration_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if ttft_ms == 0.0 {
            ttft_ms = total_duration_ms;
        }

        (
            RequestMetric {
                id: req_id,
                status_code,
                success,
                ttft_ms,
                total_duration_ms,
                chunk_count,
                bytes_received: total_bytes,
                inter_chunk_deltas_ms: inter_chunk_deltas,
                error: if success { None } else { Some(format!("HTTP {}", status_code)) },
            },
            if !chunk_samples.is_empty() { Some(chunk_samples) } else { None },
        )
    }
}
