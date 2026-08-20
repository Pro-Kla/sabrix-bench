use std::process::Command;

fn get_bin_path() -> String {
    env!("CARGO_BIN_EXE_sabrix-bench").to_string()
}

#[test]
fn test_cli_compare_default() {
    let bin = get_bin_path();
    let output = Command::new(&bin)
        .arg("compare")
        .arg("--turns")
        .arg("5")
        .output()
        .expect("Failed to run compare command");

    if !output.status.success() {
        panic!(
            "Failed with status {:?}, stderr: {}, stdout: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SABRIX MULTI-TURN AGENT LATENCY"));
    assert!(stdout.contains("Turn 1"));
    assert!(stdout.contains("EXECUTIVE SUMMARY"));
    assert!(stdout.contains("5-Turn Autonomous Agent Loop"));
}

#[test]
fn test_cli_compare_json_output() {
    let bin = get_bin_path();
    let output = Command::new(&bin)
        .arg("compare")
        .arg("--turns")
        .arg("5")
        .arg("--json")
        .output()
        .expect("Failed to run compare --json command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse output as JSON");

    assert_eq!(parsed["total_turns"], 5);
    assert!(parsed["total_in_process_ms"].is_number());
    assert_eq!(parsed["total_saas_ms"], 600.0);
    assert!(parsed["speedup_factor"].as_f64().unwrap() > 100.0);
    assert!(parsed["turns"].is_array());
    assert_eq!(parsed["turns"].as_array().unwrap().len(), 5);
}

#[test]
fn test_cli_compare_matrix() {
    let bin = get_bin_path();
    let output = Command::new(&bin)
        .arg("compare")
        .arg("--matrix")
        .output()
        .expect("Failed to run compare --matrix command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AGENT SECURITY & RUNTIME ARCHITECTURAL COMPARISON MATRIX"));
    assert!(stdout.contains("Sabrix In-VPC Gateway"));
    assert!(stdout.contains("SaaS AI Firewalls"));
}

#[test]
fn test_cli_compare_custom_latency() {
    let bin = get_bin_path();
    let output = Command::new(&bin)
        .arg("compare")
        .arg("--turns")
        .arg("3")
        .arg("--saas-latency-ms")
        .arg("80.0")
        .arg("--json")
        .output()
        .expect("Failed to run compare with custom saas latency");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse output as JSON");

    assert_eq!(parsed["total_turns"], 3);
    assert_eq!(parsed["saas_baseline_latency_ms"], 80.0);
    assert_eq!(parsed["total_saas_ms"], 240.0);
}
