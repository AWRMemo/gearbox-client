//! AI Quality A/B Validation — LlamaService (Qwen 3.5) vs FallbackService
//!
//! ## Running
//!
//! * **CI / canned** (no model required):
//!   `cargo test -p relay -- ai_quality_bench::test_quality_comparison_canned`
//!
//! * **Real model** (requires GGUF asset; ignored by default):
//!   `cargo test -p relay -- ai_quality_bench::test_quality_comparison_real_model -- --ignored`
//!
//! * **Standalone bench print-out**:
//!   `cargo test -p relay -- ai_quality_bench::bench_print_metrics -- --nocapture`

use crate::ai::llama_service::LlamaService;
use relay_core::ai::fallback::FallbackService;
use relay_core::ai::service::AIService;
use relay_core::types::{EnrichmentOutput, Highlight};

const HIGHLIGHTS: &[(&str, &str)] = &[
    ("hl-01",
        "Quantum computers exploit superposition to solve problems exponentially faster than classical machines for specific tasks like factorisation and simulation. The race to build fault-tolerant qubits has attracted billions in funding from governments and venture capital alike."),
    ("hl-02",
        "Rust's ownership system eliminates entire classes of memory bugs at compile time without needing a garbage collector. This makes it ideal for systems programming where safety and performance are both paramount."),
    ("hl-03",
        "The Roman Empire's collapse stemmed from economic overextension, military decentralisation, and political instability rather than a single cause. Historians continue to debate the relative weight of each factor."),
    ("hl-04",
        "Existentialism asserts that existence precedes essence, placing the burden of meaning-making squarely on the individual. Sartre and Camus both explored this idea, though with differing conclusions about hope and absurdity."),
    ("hl-05",
        "Frank Herbert's Dune explores ecology, politics, and religion through the lens of a desert planet that produces the most valuable substance in the universe. The saga spans millennia and multiple dynasties."),
    ("hl-06",
        "CRISPR-Cas9 allows precise editing of DNA by using a guide RNA to direct the Cas9 nuclease to a specific genomic sequence. Its discovery transformed biological research and sparked ethical debates worldwide."),
    ("hl-07",
        "Transformer architectures revolutionised NLP by replacing recurrent layers with self-attention, enabling massive parallelisation during training. GPT and BERT are the most famous descendants of this approach."),
    ("hl-08",
        "The Industrial Revolution began in Britain around 1760, driven by mechanisation of textile production and the advent of steam power. It reshaped society, moving populations from countryside to city."),
    ("hl-09",
        "Stoicism teaches that virtue is the sole good and that we should focus on what is within our control while accepting what is not. Marcus Aurelius and Epictetus are its most widely read ancient proponents."),
    ("hl-10",
        "Orwell's 1984 depicts a totalitarian regime that maintains power through pervasive surveillance, historical revisionism, and the manipulation of language. The term doublespeak entered common usage from this novel."),
    ("hl-11",
        "Black holes emit Hawking radiation due to quantum effects near the event horizon, causing them to slowly evaporate over astronomical timescales. This theoretical prediction remains extraordinarily difficult to verify experimentally."),
    ("hl-12",
        "Proof-of-stake replaces energy-intensive mining with economic staking, selecting validators based on the cryptocurrency they lock up as collateral. Ethereum's The Merge in 2022 was the largest deployment of this consensus mechanism."),
    ("hl-13",
        "The Cold War was a decades-long geopolitical standoff between the United States and the Soviet Union, characterised by proxy wars, an arms race, and ideological competition. It shaped the modern world order more than any other twentieth-century conflict."),
    ("hl-14",
        "Utilitarianism holds that the morally right action is the one that produces the greatest happiness for the greatest number of people. Critics argue it can justify sacrificing individual rights for collective benefit."),
    ("hl-15",
        "William Gibson's Neuromancer launched the cyberpunk genre with a hacker protagonist navigating a reality saturated with artificial intelligence and corporate power. The novel coined the term cyberspace."),
    ("hl-16",
        "Photosynthesis converts light energy into chemical energy by splitting water and fixing carbon dioxide into glucose within the chloroplasts of plant cells. This process underpins nearly all life on Earth."),
    ("hl-17",
        "Kubernetes orchestrates containerised applications across a cluster, automating deployment, scaling, and self-healing through declarative configuration. It has become the de facto standard for cloud-native infrastructure."),
    ("hl-18",
        "The Renaissance saw a revival of classical learning and values in Europe, producing advances in art, science, and humanist philosophy between the fourteenth and seventeenth centuries. Figures such as Leonardo and Machiavelli defined the era."),
    ("hl-19",
        "The free-will debate centres on whether human choices are determined by prior causes or whether agents possess the genuine ability to do otherwise. Compatibilism attempts to reconcile determinism with moral responsibility."),
    ("hl-20",
        "Asimov's Foundation series applies psychohistory, a fictional statistical science, to predict and guide the future of a crumbling galactic empire. The trilogy influenced generations of economists and technologists."),
];

fn make_highlight(id: &str, text: &str) -> Highlight {
    Highlight {
        id: id.to_string(),
        text: text.to_string(),
        source_url: None,
        source_title: None,
        source_author: None,
    }
}

// ==========================
// Canned Qwen loader (CI-safe)
// ==========================

fn load_canned_responses() -> Vec<EnrichmentOutput> {
    static JSON: &str = include_str!("../fixtures/qwen_canned_responses.json");
    serde_json::from_str::<Vec<EnrichmentOutput>>(JSON).expect("fixture is valid JSON")
}

// ==========================
// Metrics helpers
// ==========================

struct QualityMetrics {
    parse_yield: f64,      // % of outputs that parsed (Qwen side)
    parse_failures: usize, // raw count
    avg_tags_qwen: f64,
    avg_tags_fallback: f64,
    avg_summary_len_qwen: f64,
    avg_summary_len_fallback: f64,
    tag_overlap_ratio: f64, // |intersection| / |union| averaged
}

fn compute_metrics(
    qwen_results: &[Result<EnrichmentOutput, String>],
    fallback_results: &[EnrichmentOutput],
) -> QualityMetrics {
    assert_eq!(qwen_results.len(), fallback_results.len());
    let total = qwen_results.len();
    let mut successes = 0usize;
    let mut tag_sum_qwen = 0usize;
    let mut tag_sum_fallback = 0usize;
    let mut summary_len_qwen = 0usize;
    let mut summary_len_fallback = 0usize;
    let mut overlap_sum = 0f64;

    let empty_enrichment = EnrichmentOutput {
        summary: String::new(),
        tags: vec![],
        connection_suggestion: None,
    };
    for (q_res, f) in qwen_results.iter().zip(fallback_results.iter()) {
        if q_res.is_ok() {
            successes += 1;
        }
        let q = q_res.as_ref().unwrap_or(&empty_enrichment);

        tag_sum_qwen += q.tags.len();
        tag_sum_fallback += f.tags.len();
        summary_len_qwen += q.summary.len();
        summary_len_fallback += f.summary.len();

        let q_set: std::collections::HashSet<&str> = q.tags.iter().map(|s| s.as_str()).collect();
        let f_set: std::collections::HashSet<&str> = f.tags.iter().map(|s| s.as_str()).collect();
        let union: std::collections::HashSet<&str> = q_set.union(&f_set).copied().collect();
        let intersection: std::collections::HashSet<&str> =
            q_set.intersection(&f_set).copied().collect();
        overlap_sum += if union.is_empty() {
            0.0
        } else {
            intersection.len() as f64 / union.len() as f64
        };
    }

    let n = total as f64;
    QualityMetrics {
        parse_yield: (successes as f64) / n,
        parse_failures: total - successes,
        avg_tags_qwen: (tag_sum_qwen as f64) / n,
        avg_tags_fallback: (tag_sum_fallback as f64) / n,
        avg_summary_len_qwen: (summary_len_qwen as f64) / n,
        avg_summary_len_fallback: (summary_len_fallback as f64) / n,
        tag_overlap_ratio: overlap_sum / n,
    }
}

// ==========================
// Tests
// ==========================

#[test]
fn test_quality_comparison_canned() {
    let canned = load_canned_responses();
    assert_eq!(
        canned.len(),
        HIGHLIGHTS.len(),
        "fixture must contain one response per highlight"
    );

    let fallback = FallbackService;
    let mut fallback_results = Vec::with_capacity(HIGHLIGHTS.len());
    for (id, text) in HIGHLIGHTS {
        let h = make_highlight(id, text);
        let out = fallback.enrich(&h, &[]).expect("fallback never fails");
        fallback_results.push(out);
    }

    let qwen_results: Vec<Result<EnrichmentOutput, String>> = canned.into_iter().map(Ok).collect();

    let m = compute_metrics(&qwen_results, &fallback_results);

    println!("\n=== Canned Qwen vs Fallback Metrics ===");
    println!("Parse yield:        {:.1}%", m.parse_yield * 100.0);
    println!("Parse failures:     {}", m.parse_failures);
    println!("Avg tags (Qwen):    {:.2}", m.avg_tags_qwen);
    println!("Avg tags (Fallback):{:.2}", m.avg_tags_fallback);
    println!("Avg summary chars (Qwen):    {:.1}", m.avg_summary_len_qwen);
    println!(
        "Avg summary chars (Fallback):{:.1}",
        m.avg_summary_len_fallback
    );
    println!("Tag overlap ratio:  {:.2}", m.tag_overlap_ratio);

    // Guard-rail assertions based on fixture
    assert!(
        (m.parse_yield - 1.0).abs() < f64::EPSILON,
        "canned data must parse at 100%"
    );
    assert!(
        m.avg_tags_qwen >= 3.0,
        "Qwen must produce at least 3 tags on average"
    );
    assert!(
        m.avg_summary_len_qwen >= 50.0,
        "Qwen summaries must be reasonably substantive"
    );
}

#[test]
fn test_qwen_canned_produces_connection_null() {
    // Spot-check: all 20 canned responses have connection: null because we
    // supplied empty candidates. Ensure parser handled it correctly.
    let canned = load_canned_responses();
    for out in &canned {
        assert!(
            out.connection_suggestion.is_none(),
            "connection should be None when no candidates are provided"
        );
    }
}

#[test]
#[ignore = "requires Qwen-3.5-0.8B GGUF model asset present at the path returned by model_manager::ensure_model()"]
fn test_quality_comparison_real_model() {
    // This test loads the real model via LlamaService::try_load and runs the
    // same 20 highlights. It is ignored by default because the model is a
    // multi-hundred-megabyte binary that is not stored in git.
    //
    // To run locally:
    //   1. Ensure the model has been downloaded (via model_manager::ensure_model).
    //   2. cargo test -p relay -- ai_quality_bench::test_quality_comparison_real_model -- --ignored
    use std::path::PathBuf;

    let model_path = std::env::var("GEARBOX_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Default fallback for local dev where ensure_model puts the asset
            let dir = std::env::temp_dir();
            dir.join("gearbox_models").join("qwen-3.5-0.8b.gguf")
        });

    if !model_path.exists() {
        panic!(
            "Model not found at {}. Set GEARBOX_MODEL_PATH or download the model.",
            model_path.display()
        );
    }

    let ai = LlamaService::try_load(&model_path).expect("model load failed");
    let fallback = FallbackService;

    let mut qwen_results = Vec::with_capacity(HIGHLIGHTS.len());
    let mut fallback_results = Vec::with_capacity(HIGHLIGHTS.len());

    for (id, text) in HIGHLIGHTS {
        let h = make_highlight(id, text);
        let q = ai.enrich(&h, &[]);
        let f = fallback.enrich(&h, &[]).expect("fallback never fails");
        qwen_results.push(q);
        fallback_results.push(f);
    }

    let m = compute_metrics(&qwen_results, &fallback_results);

    println!("\n=== Real Model Qwen vs Fallback Metrics ===");
    println!("Parse yield:        {:.1}%", m.parse_yield * 100.0);
    println!("Parse failures:     {}", m.parse_failures);
    println!("Avg tags (Qwen):    {:.2}", m.avg_tags_qwen);
    println!("Avg tags (Fallback):{:.2}", m.avg_tags_fallback);
    println!("Avg summary chars (Qwen):    {:.1}", m.avg_summary_len_qwen);
    println!(
        "Avg summary chars (Fallback):{:.1}",
        m.avg_summary_len_fallback
    );
    println!("Tag overlap ratio:  {:.2}", m.tag_overlap_ratio);

    assert!(
        m.parse_yield >= 0.80,
        "Qwen parse yield must be >= 80% for production. Actual: {:.1}%",
        m.parse_yield * 100.0
    );
    assert!(
        m.avg_tags_qwen >= 2.0,
        "Qwen must produce at least 2 tags on average"
    );
}

/// Print-only bench useful for human spot-checks.
#[test]
fn bench_print_metrics() {
    let canned = load_canned_responses();
    let fallback = FallbackService;
    let mut fallback_results = Vec::with_capacity(HIGHLIGHTS.len());
    for (id, text) in HIGHLIGHTS {
        let h = make_highlight(id, text);
        let out = fallback.enrich(&h, &[]).expect("fallback never fails");
        fallback_results.push(out);
    }
    let qwen_results: Vec<Result<EnrichmentOutput, String>> = canned.into_iter().map(Ok).collect();

    let m = compute_metrics(&qwen_results, &fallback_results);

    println!("\n=== A/B Quality Summary ===");
    println!("Sample count:       {}", HIGHLIGHTS.len());
    println!("Parse yield (Qwen): {:.1}%", m.parse_yield * 100.0);
    println!("Parse failures:     {}", m.parse_failures);
    println!("Avg tags (Qwen):    {:.2}", m.avg_tags_qwen);
    println!("Avg tags (Fallback):{:.2}", m.avg_tags_fallback);
    println!("Avg summary chars (Qwen):    {:.1}", m.avg_summary_len_qwen);
    println!(
        "Avg summary chars (Fallback):{:.1}",
        m.avg_summary_len_fallback
    );
    println!("Tag overlap ratio:  {:.2}", m.tag_overlap_ratio);
    println!("\n--- Spot-check highlights ---");
    for i in [0, 4, 9, 14, 19] {
        let (id, text) = HIGHLIGHTS[i];
        let q = &qwen_results[i].as_ref().unwrap();
        let f = &fallback_results[i];
        println!("\n[{id}] {text:.80}...");
        println!("  Qwen tags:    {:?}", q.tags);
        println!("  Fallback tags:{:?}", f.tags);
        println!(
            "  Qwen summary:     {}",
            &q.summary[..q.summary.len().min(100)]
        );
        println!(
            "  Fallback summary: {}",
            &f.summary[..f.summary.len().min(100)]
        );
    }
}
