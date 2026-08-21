use anyhow::{Context, Result};
use serde_json::Value;

pub const RAG_SUITE_JSON: &str = include_str!("rag.json");
pub const OWASP_SUITE_JSON: &str = include_str!("owasp.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSuite {
    Rag,
    Owasp,
    Simple,
}

impl BenchmarkSuite {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rag" | "enterprise" | "throughput" => Some(Self::Rag),
            "owasp" | "security" | "safety" => Some(Self::Owasp),
            "simple" | "default" => Some(Self::Simple),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Rag => "Enterprise RAG Suite (50 Prompts)",
            Self::Owasp => "OWASP LLM Top-10 & Safety Suite (50 Probes)",
            Self::Simple => "Simple Ping / TTFT Baseline Suite (10 Prompts)",
        }
    }

    pub fn load_payloads(&self) -> Result<Vec<Value>> {
        match self {
            Self::Rag => serde_json::from_str(RAG_SUITE_JSON)
                .context("Failed to parse embedded RAG test suite JSON"),
            Self::Owasp => serde_json::from_str(OWASP_SUITE_JSON)
                .context("Failed to parse embedded OWASP test suite JSON"),
            Self::Simple => {
                let simple_json = r#"[
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Ping"}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Hello, world!"}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Compute 2 + 2"}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Explain the word 'throughput' in 5 words."}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "What is the capital of France?"}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Output the first 3 prime numbers."}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "What is the speed of light in vacuum?"}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Return 'OK'"}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Define SSE (Server-Sent Events) in one sentence."}], "stream": true},
                    {"model": "gpt-4o", "messages": [{"role": "user", "content": "Say 'ready' to confirm connection."}], "stream": true}
                ]"#;
                serde_json::from_str(simple_json)
                    .context("Failed to parse simple baseline test suite")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_suite_loads_50_prompts() {
        let suite = BenchmarkSuite::Rag;
        let payloads = suite.load_payloads().expect("RAG suite should parse");
        assert_eq!(payloads.len(), 50);
        assert_eq!(payloads[0]["id"], "rag-01");
    }

    #[test]
    fn test_owasp_suite_loads_50_probes() {
        let suite = BenchmarkSuite::Owasp;
        let payloads = suite.load_payloads().expect("OWASP suite should parse");
        assert_eq!(payloads.len(), 50);
        assert_eq!(payloads[0]["id"], "owasp-01");
    }

    #[test]
    fn test_simple_suite_loads_10_prompts() {
        let suite = BenchmarkSuite::Simple;
        let payloads = suite.load_payloads().expect("Simple suite should parse");
        assert_eq!(payloads.len(), 10);
    }
}
