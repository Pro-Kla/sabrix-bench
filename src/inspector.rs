use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Safe => write!(f, "SAFE"),
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFinding {
    pub rule_id: String,
    pub level: RiskLevel,
    pub title: String,
    pub details: String,
    pub matched_snippet: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub raw_json: String,
    pub is_request: bool,
    pub method: String,
    pub tool_name: Option<String>,
    pub arguments: Option<Value>,
    pub findings: Vec<RiskFinding>,
    pub max_risk_level: RiskLevel,
    pub parse_duration_us: f64,
    pub inspection_duration_us: f64,
    pub total_duration_us: f64,
    pub payload_bytes: usize,
}

pub struct McpInspector;

impl McpInspector {
    /// Inspects a raw JSON-RPC string, measuring parse and inspection latency in microseconds.
    pub fn inspect_json_str(raw_json: &str) -> Result<InspectionResult> {
        let payload_bytes = raw_json.len();
        let t0 = Instant::now();

        // 1. JSON parsing phase
        let parsed_val: Value =
            serde_json::from_str(raw_json).context("Failed to parse input as valid JSON")?;
        let t1 = Instant::now();
        let parse_duration_us = (t1 - t0).as_secs_f64() * 1_000_000.0;

        // 2. Deterministic Rule Inspection Phase
        let mut findings = Vec::new();
        let mut tool_name = None;
        let mut arguments = None;

        let (is_request, method) =
            if let Some(m) = parsed_val.get("method").and_then(|v| v.as_str()) {
                let m_str = m.to_string();
                if m_str == "tools/call" || m_str == "tool/call" {
                    if let Some(params) = parsed_val.get("params") {
                        if let Some(name) = params.get("name").and_then(|v| v.as_str()) {
                            tool_name = Some(name.to_string());
                        }
                        if let Some(args) = params.get("arguments") {
                            arguments = Some(args.clone());
                        }
                    }
                } else if m_str == "tools/list" {
                    tool_name = Some("<list_tools>".to_string());
                } else if m_str == "resources/read" {
                    if let Some(params) = parsed_val.get("params") {
                        if let Some(uri) = params.get("uri").and_then(|v| v.as_str()) {
                            tool_name = Some(format!("resource:{}", uri));
                        }
                    }
                }
                (true, m_str)
            } else if parsed_val.get("result").is_some() || parsed_val.get("error").is_some() {
                (false, "jsonrpc/response".to_string())
            } else {
                (false, "jsonrpc/unknown".to_string())
            };

        // Run security checks on entire JSON string and arguments
        Self::evaluate_security_rules(raw_json, &parsed_val, &mut findings);

        let t2 = Instant::now();
        let inspection_duration_us = (t2 - t1).as_secs_f64() * 1_000_000.0;
        let total_duration_us = (t2 - t0).as_secs_f64() * 1_000_000.0;

        let max_risk_level = findings
            .iter()
            .map(|f| f.level)
            .max()
            .unwrap_or(RiskLevel::Safe);

        Ok(InspectionResult {
            raw_json: raw_json.to_string(),
            is_request,
            method,
            tool_name,
            arguments,
            findings,
            max_risk_level,
            parse_duration_us,
            inspection_duration_us,
            total_duration_us,
            payload_bytes,
        })
    }

    fn evaluate_security_rules(raw_json: &str, parsed: &Value, findings: &mut Vec<RiskFinding>) {
        let text_lower = raw_json.to_lowercase();

        // Rule 1: Destructive File System Operations
        let dangerous_fs_patterns = [
            (
                "rm -rf",
                RiskLevel::Critical,
                "Recursive forced file deletion",
            ),
            (
                "rmdir /s",
                RiskLevel::Critical,
                "Windows recursive directory removal",
            ),
            ("mkfs", RiskLevel::Critical, "Filesystem formatting command"),
            ("dd if=", RiskLevel::Critical, "Raw disk block overwrite"),
            (
                "chmod 777",
                RiskLevel::High,
                "Unsafe global read/write/execute permissions",
            ),
            (
                "chmod -r 777",
                RiskLevel::Critical,
                "Recursive unsafe global permissions",
            ),
            (
                ":(){ :|:& };:",
                RiskLevel::Critical,
                "Fork bomb shell explosion pattern",
            ),
            (
                "shutdown -h",
                RiskLevel::High,
                "System shutdown instruction",
            ),
        ];

        for (pattern, level, desc) in dangerous_fs_patterns {
            if text_lower.contains(pattern) {
                findings.push(RiskFinding {
                    rule_id: "MCP-SEC-001".to_string(),
                    level,
                    title: "Destructive Shell Command Detected".to_string(),
                    details: format!(
                        "Found destructive filesystem signature '{}': {}",
                        pattern, desc
                    ),
                    matched_snippet: pattern.to_string(),
                });
            }
        }

        // Rule 2: Remote Code Execution & Unsafe Shell Pipes
        let rce_patterns = [
            (
                "curl | sh",
                RiskLevel::Critical,
                "Piping remote script directly into shell",
            ),
            (
                "curl | bash",
                RiskLevel::Critical,
                "Piping remote script directly into bash",
            ),
            (
                "wget | bash",
                RiskLevel::Critical,
                "Piping remote script directly into bash",
            ),
            (
                "wget | sh",
                RiskLevel::Critical,
                "Piping remote script directly into shell",
            ),
            (
                "nc -e",
                RiskLevel::Critical,
                "Netcat reverse shell spawn pattern",
            ),
            (
                "/dev/tcp/",
                RiskLevel::Critical,
                "Bash socket reverse shell redirection",
            ),
            (
                "powershell -enc",
                RiskLevel::High,
                "Encoded PowerShell payload execution",
            ),
        ];

        for (pattern, level, desc) in rce_patterns {
            let normalized = text_lower.replace(" ", "");
            let pat_normalized = pattern.replace(" ", "");
            if text_lower.contains(pattern) || normalized.contains(&pat_normalized) {
                findings.push(RiskFinding {
                    rule_id: "MCP-SEC-002".to_string(),
                    level,
                    title: "Remote Execution / Reverse Shell Pipe".to_string(),
                    details: desc.to_string(),
                    matched_snippet: pattern.to_string(),
                });
            }
        }

        // Rule 3: Destructive SQL Queries & Injection Mutations
        let dangerous_sql = [
            (
                "drop table",
                RiskLevel::Critical,
                "Irreversible SQL table drop",
            ),
            (
                "drop database",
                RiskLevel::Critical,
                "Irreversible SQL database drop",
            ),
            ("truncate table", RiskLevel::Critical, "Full table wipe"),
            (
                "delete from",
                RiskLevel::High,
                "Unconstrained or mass row deletion",
            ),
            (
                "alter table",
                RiskLevel::Medium,
                "Schema alteration mutation",
            ),
            (
                "where 1=1",
                RiskLevel::High,
                "Tautological SQL bypass predicate",
            ),
            (
                "information_schema",
                RiskLevel::Medium,
                "Database schema enumeration probe",
            ),
        ];

        for (pattern, level, desc) in dangerous_sql {
            if text_lower.contains(pattern) {
                findings.push(RiskFinding {
                    rule_id: "MCP-SEC-003".to_string(),
                    level,
                    title: "Dangerous SQL Query / Schema Mutation".to_string(),
                    details: format!("Query contains '{}': {}", pattern, desc),
                    matched_snippet: pattern.to_string(),
                });
            }
        }

        // Rule 4: Credential & API Key Leakage
        if let Some(idx) = raw_json.find("sk-") {
            let candidate: String = raw_json[idx..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if candidate.len() >= 16 {
                findings.push(RiskFinding {
                    rule_id: "MCP-SEC-004".to_string(),
                    level: RiskLevel::Critical,
                    title: "Exposed OpenAI / Provider API Key".to_string(),
                    details: "Unmasked secret key found in tool parameters or payload".to_string(),
                    matched_snippet: format!(
                        "sk-{}...",
                        &candidate[3..std::cmp::min(10, candidate.len())]
                    ),
                });
            }
        }

        if raw_json.contains("ghp_") {
            findings.push(RiskFinding {
                rule_id: "MCP-SEC-005".to_string(),
                level: RiskLevel::Critical,
                title: "Exposed GitHub Personal Access Token".to_string(),
                details: "Unmasked GitHub PAT detected in MCP arguments".to_string(),
                matched_snippet: "ghp_***".to_string(),
            });
        }

        if raw_json.contains("AKIA") {
            findings.push(RiskFinding {
                rule_id: "MCP-SEC-006".to_string(),
                level: RiskLevel::High,
                title: "Exposed AWS Access Key ID".to_string(),
                details: "AWS IAM credential identifier detected in JSON payload".to_string(),
                matched_snippet: "AKIA***".to_string(),
            });
        }

        if raw_json.contains("-----BEGIN") && raw_json.contains("PRIVATE KEY-----") {
            findings.push(RiskFinding {
                rule_id: "MCP-SEC-007".to_string(),
                level: RiskLevel::Critical,
                title: "Private Cryptographic Key Exfiltration".to_string(),
                details: "Raw PEM private key discovered in transit".to_string(),
                matched_snippet: "-----BEGIN PRIVATE KEY-----".to_string(),
            });
        }

        // Rule 5: Sensitive Path Egress
        let sensitive_paths = [
            ("/etc/passwd", RiskLevel::High, "System user database read"),
            (
                "/etc/shadow",
                RiskLevel::Critical,
                "System shadow password hash read",
            ),
            (
                ".ssh/id_rsa",
                RiskLevel::Critical,
                "SSH private identity read",
            ),
            (
                ".ssh/id_ed25519",
                RiskLevel::Critical,
                "SSH private identity read",
            ),
            (
                ".aws/credentials",
                RiskLevel::Critical,
                "Local AWS credentials file read",
            ),
            (
                ".env",
                RiskLevel::Medium,
                "Environment variable file access",
            ),
        ];

        for (path, level, desc) in sensitive_paths {
            if text_lower.contains(path) {
                findings.push(RiskFinding {
                    rule_id: "MCP-SEC-008".to_string(),
                    level,
                    title: "Sensitive Local Path Reference".to_string(),
                    details: format!(
                        "Detected access to protected system path '{}': {}",
                        path, desc
                    ),
                    matched_snippet: path.to_string(),
                });
            }
        }

        // Rule 6: Unconstrained System Execution Tool
        if let Some(params) = parsed.get("params") {
            if let Some(tool) = params.get("name").and_then(|v| v.as_str()) {
                let tool_lower = tool.to_lowercase();
                if tool_lower == "execute_command"
                    || tool_lower == "bash"
                    || tool_lower == "sh"
                    || tool_lower == "run_terminal"
                {
                    findings.push(RiskFinding {
                        rule_id: "MCP-SEC-009".to_string(),
                        level: RiskLevel::Medium,
                        title: "Arbitrary Shell Execution Primitive".to_string(),
                        details: format!("Agent invoked unconstrained execution tool '{}'", tool),
                        matched_snippet: tool.to_string(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_mcp_call() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "get_weather",
                "arguments": { "city": "San Francisco" }
            }
        }"#;

        let res = McpInspector::inspect_json_str(json).unwrap();
        assert_eq!(res.method, "tools/call");
        assert_eq!(res.tool_name.as_deref(), Some("get_weather"));
        assert_eq!(res.max_risk_level, RiskLevel::Safe);
        assert!(res.findings.is_empty());
        assert!(res.total_duration_us > 0.0);
    }

    #[test]
    fn test_destructive_rm_rf() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/call",
            "params": {
                "name": "execute_command",
                "arguments": { "command": "rm -rf /var/log/*" }
            }
        }"#;

        let res = McpInspector::inspect_json_str(json).unwrap();
        assert_eq!(res.max_risk_level, RiskLevel::Critical);
        assert!(res.findings.iter().any(|f| f.rule_id == "MCP-SEC-001"));
    }

    #[test]
    fn test_sql_drop_table() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "query-9",
            "method": "tools/call",
            "params": {
                "name": "database_query",
                "arguments": { "sql": "DROP TABLE users; --" }
            }
        }"#;

        let res = McpInspector::inspect_json_str(json).unwrap();
        assert_eq!(res.max_risk_level, RiskLevel::Critical);
        assert!(res.findings.iter().any(|f| f.rule_id == "MCP-SEC-003"));
    }

    #[test]
    fn test_openai_api_key_leak() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": {
                "name": "fetch_api",
                "arguments": { "header": "Bearer sk-proj-1234567890abcdef1234567890" }
            }
        }"#;

        let res = McpInspector::inspect_json_str(json).unwrap();
        assert_eq!(res.max_risk_level, RiskLevel::Critical);
        assert!(res.findings.iter().any(|f| f.rule_id == "MCP-SEC-004"));
    }

    #[test]
    fn test_sensitive_path_access() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "read_file",
                "arguments": { "path": "/etc/passwd" }
            }
        }"#;

        let res = McpInspector::inspect_json_str(json).unwrap();
        assert_eq!(res.max_risk_level, RiskLevel::High);
        assert!(res.findings.iter().any(|f| f.rule_id == "MCP-SEC-008"));
    }
}
