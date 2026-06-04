use serde::{Deserialize, Serialize};
use std::time::Instant;

/// The model backend under test.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    DeterministicFallback,
    LlamaCpp {
        model_path: String,
    },
    #[cfg(test)]
    Mock {
        latency_ms: u64,
    },
}

/// A single benchmark run configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub name: String,
    pub backend: Backend,
    pub prompts: Vec<String>,
    pub warm_up_count: usize,
    pub iterations: usize,
}

/// Per-prompt latency and token estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    pub prompt: String,
    pub latency_ms: f64,
    pub tokens_per_sec: f64,
    pub output_len_chars: usize,
}

/// Results for a single backend run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub name: String,
    pub backend: String,
    pub total_duration_ms: f64,
    pub avg_latency_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub prompts: Vec<PromptResult>,
    pub summary_score: f64,
}

impl RunResult {
    pub fn compute_percentiles(&mut self) {
        let mut latencies: Vec<f64> = self.prompts.iter().map(|p| p.latency_ms).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if !latencies.is_empty() {
            self.p50_ms = percentile(&latencies, 0.50);
            self.p95_ms = percentile(&latencies, 0.95);
            self.p99_ms = percentile(&latencies, 0.99);
        }
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len() - 1) as f64 * p).floor() as usize;
    sorted[idx.clamp(0, sorted.len() - 1)]
}

/// Use a `FnMut` closure to measure a single enrichment operation.
/// Mirrors `relay_core::ai::service::AIService::enrich` but stripped of candidates.
pub fn measure_one<F>(mut f: F, prompt: &str) -> PromptResult
where
    F: FnMut(&str) -> Result<String, String>,
{
    let start = Instant::now();
    let output = f(prompt);
    let elapsed = start.elapsed();

    let ms = elapsed.as_secs_f64() * 1000.0;
    let out_len = output.as_ref().map(|s| s.len()).unwrap_or(0);

    // Rough token estimate: ~4 chars per token for English text.
    let tokens_est = out_len.max(1) as f64 / 4.0;
    let tps = (tokens_est / elapsed.as_secs_f64()).max(0.0);

    PromptResult {
        prompt: prompt.to_string(),
        latency_ms: ms,
        tokens_per_sec: tps,
        output_len_chars: out_len,
    }
}

/// Aggregate a batch of results into a `RunResult`.
pub fn aggregate(name: String, backend: String, results: Vec<PromptResult>) -> RunResult {
    let total_ms: f64 = results.iter().map(|r| r.latency_ms).sum();
    let avg_ms = total_ms / results.len().max(1) as f64;
    let score = results.iter().map(|r| r.tokens_per_sec).sum::<f64>() / results.len().max(1) as f64;
    let mut run = RunResult {
        name,
        backend,
        total_duration_ms: total_ms,
        avg_latency_ms: avg_ms,
        p50_ms: 0.0,
        p95_ms: 0.0,
        p99_ms: 0.0,
        prompts: results,
        summary_score: score,
    };
    run.compute_percentiles();
    run
}

/// Default benchmark prompts covering a range of inputs.
pub fn default_prompts() -> Vec<String> {
    vec![
        "Rust is a systems programming language that runs blazingly fast.".to_string(),
        "The quick brown fox jumps over the lazy dog. This sentence is a pangram, containing every letter of the English alphabet at least once.".to_string(),
        "Large language models are transformer-based neural networks trained on vast corpora. They exhibit emergent capabilities at scale, such as in-context learning and chain-of-thought reasoning. These properties make them powerful general-purpose AI assistants but also raise questions about hallucination and alignment.".to_string(),
        "q".repeat(500), // edge-case: repetitive input
        "A".repeat(8000), // edge-case: near-max input
    ]
}
