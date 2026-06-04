use relay_core::ai::fallback::FallbackService;
use relay_core::ai::service::AIService;
use relay_core::types::Highlight;
use std::fs;
use std::path::Path;

mod types;
use types::{aggregate, default_prompts, measure_one, Backend, BenchmarkRun};

fn main() {
    let run = parse_args_or_default();
    println!("relay-bench  v{}", env!("CARGO_PKG_VERSION"));
    println!("Run: {}", run.name);
    println!("Prompts: {}", run.prompts.len());
    println!("Iterations per prompt: {}", run.iterations);
    println!("Warm-up iterations: {}", run.warm_up_count);
    println!("---");

    match run.backend {
        Backend::DeterministicFallback => bench_fallback(&run),
        Backend::LlamaCpp { model_path } => {
            eprintln!(
                "LlamaCpp backend not yet implemented. Model path: {}",
                model_path
            );
            std::process::exit(1);
        }
        #[cfg(test)]
        Backend::Mock { latency_ms } => bench_mock(&run, latency_ms),
    }
}

fn parse_args_or_default() -> BenchmarkRun {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "--config" if args.len() >= 3 => {
                let path = &args[2];
                let yaml = fs::read_to_string(path).expect("Failed to read config file");
                return serde_yaml::from_str(&yaml).expect("Invalid YAML config");
            }
            "--model" if args.len() >= 3 => {
                let model_name = args[2].clone();
                let name = args.get(3).cloned().unwrap_or_else(|| model_name.clone());
                return BenchmarkRun {
                    name,
                    backend: Backend::LlamaCpp {
                        model_path: model_name,
                    },
                    prompts: default_prompts(),
                    warm_up_count: 2,
                    iterations: 5,
                };
            }
            "--compare" if args.len() >= 4 => {
                let model_a = args[2].clone();
                let model_b = args[3].clone();
                eprintln!("Compare mode: A={model_a}, B={model_b}");
                eprintln!("Run each model separately with --model, then diff the YAML results.");
                std::process::exit(0);
            }
            _ => {}
        }
    }
    BenchmarkRun {
        name: "default".to_string(),
        backend: Backend::DeterministicFallback,
        prompts: default_prompts(),
        warm_up_count: 10,
        iterations: 100,
    }
}

fn bench_fallback(run: &BenchmarkRun) {
    let service = FallbackService;

    // Warm-up
    for prompt in run.prompts.iter().take(run.warm_up_count.max(1)) {
        let h = make_highlight(prompt);
        let _ = service.enrich(&h, &[]);
    }

    let mut all_results = Vec::new();
    for _ in 0..run.iterations {
        let mut batch = Vec::new();
        for prompt in &run.prompts {
            let result = measure_one(
                |text| {
                    let hl = make_highlight(text);
                    let out = service.enrich(&hl, &[])?;
                    Ok(serde_json::to_string(&out).unwrap_or_default().to_string())
                },
                prompt,
            );
            batch.push(result);
        }
        all_results.push(batch);
    }

    // Flatten all prompts across all iterations
    let flat: Vec<types::PromptResult> = all_results.into_iter().flatten().collect();
    let mut run_result = aggregate(run.name.clone(), "deterministic_fallback".to_string(), flat);
    run_result.compute_percentiles();

    // Print human-readable summary
    println!("\n=== Results: {} ===", run_result.name);
    println!("Backend: {}", run_result.backend);
    println!("Total time: {:.2} ms", run_result.total_duration_ms);
    println!("Avg latency: {:.2} ms", run_result.avg_latency_ms);
    println!("p50: {:.2} ms", run_result.p50_ms);
    println!("p95: {:.2} ms", run_result.p95_ms);
    println!("p99: {:.2} ms", run_result.p99_ms);
    println!("Score (tok/s): {:.2}", run_result.summary_score);
    println!("---");
    for pr in &run_result.prompts[..(run_result.prompts.len().min(3))] {
        println!(
            "Prompt ({} chars): latency={:.2}ms tps={:.2}",
            pr.prompt.len().min(80),
            pr.latency_ms,
            pr.tokens_per_sec
        );
    }

    // Write YAML report to relay-bench/results/
    let results_dir = Path::new("relay-bench/results");
    let _ = fs::create_dir_all(results_dir);
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_path = results_dir.join(format!("{}_{}.yaml", run_result.name, timestamp));
    let yaml = serde_yaml::to_string(&run_result).expect("Failed to serialize results");
    fs::write(&report_path, yaml).expect("Failed to write report");
    println!("Report written to {}", report_path.display());
}

#[cfg(test)]
fn bench_mock(run: &BenchmarkRun, latency_ms: u64) {
    let mut all_results = Vec::new();
    for prompt in &run.prompts {
        let result = measure_one(
            |_| {
                std::thread::sleep(std::time::Duration::from_millis(latency_ms));
                Ok("mock output".to_string())
            },
            prompt,
        );
        all_results.push(result);
    }
    let mut run_result = aggregate(run.name.clone(), "mock".to_string(), all_results);
    run_result.compute_percentiles();
    println!("Mock backend: avg={:.2}ms", run_result.avg_latency_ms);
}

fn make_highlight(text: &str) -> Highlight {
    Highlight {
        id: "bench-id".to_string(),
        text: text.to_string(),
        source_url: None,
        source_title: None,
        source_author: None,
    }
}
