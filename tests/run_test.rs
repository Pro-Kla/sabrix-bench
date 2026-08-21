use std::process::Command;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_mock_sse_gateway_benchmark() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let target_url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

    // Background server task
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 1024];
                let _ = socket.read(&mut buf).await;

                let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n";
                let _ = socket.write_all(headers.as_bytes()).await;

                tokio::time::sleep(Duration::from_millis(2)).await;
                let chunk1 = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
                let _ = socket.write_all(chunk1.as_bytes()).await;

                tokio::time::sleep(Duration::from_millis(2)).await;
                let chunk2 = "data: {\"choices\":[{\"delta\":{\"content\":\" world!\"}}]}\n\n";
                let _ = socket.write_all(chunk2.as_bytes()).await;

                let chunk3 = "data: [DONE]\n\n";
                let _ = socket.write_all(chunk3.as_bytes()).await;
                let _ = socket.flush().await;
                drop(socket);
            });
        }
    });

    let temp_dir = std::env::temp_dir();
    let html_report = temp_dir.join("test_sabrix_report.html");
    let json_report = temp_dir.join("test_sabrix_report.json");

    let bin = env!("CARGO_BIN_EXE_sabrix-bench").to_string();
    let target_clone = target_url.clone();
    let html_clone = html_report.to_str().unwrap().to_string();
    let json_clone = json_report.to_str().unwrap().to_string();

    let output = tokio::task::spawn_blocking(move || {
        Command::new(bin)
            .args([
                "run",
                "--target",
                &target_clone,
                "--concurrency",
                "2",
                "--requests",
                "4",
                "--suite",
                "simple",
                "--export-html",
                &html_clone,
                "--export-json",
                &json_clone,
            ])
            .output()
            .expect("Failed to execute sabrix-bench run")
    })
    .await
    .unwrap();

    assert!(
        output.status.success(),
        "sabrix-bench run failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("⚡ SABRIX AI PROXY & LLM GATEWAY BENCHMARK REPORT"));
    assert!(stdout.contains("Time-to-First-Token (TTFT)"));
    assert!(stdout.contains("Throughput (req/s)"));

    assert!(html_report.exists());
    assert!(json_report.exists());

    let html_content = std::fs::read_to_string(&html_report).unwrap();
    assert!(html_content.contains("Sabrix Benchmark Report"));
    assert!(html_content.contains("Latency Percentile Distribution"));

    let json_content = std::fs::read_to_string(&json_report).unwrap();
    assert!(json_content.contains("\"target_url\""));
    assert!(json_content.contains("\"ttft_stats_ms\""));

    let _ = std::fs::remove_file(html_report);
    let _ = std::fs::remove_file(json_report);
}
