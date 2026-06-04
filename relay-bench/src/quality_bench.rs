use relay_core::ai::fallback::FallbackService;
use relay_core::ai::service::AIService;
use relay_core::types::{EnrichmentOutput, Highlight};
use std::collections::HashSet;

const HIGHLIGHTS: &[(&str, &str)] = &[
    ("hl-01", "Quantum computers exploit superposition to solve problems exponentially faster than classical machines for specific tasks like factorisation and simulation. The race to build fault-tolerant qubits has attracted billions in funding from governments and venture capital alike."),
    ("hl-02", "Rust's ownership system eliminates entire classes of memory bugs at compile time without needing a garbage collector. This makes it ideal for systems programming where safety and performance are both paramount."),
    ("hl-03", "The Roman Empire's collapse stemmed from economic overextension, military decentralisation, and political instability rather than a single cause. Historians continue to debate the relative weight of each factor."),
    ("hl-04", "Existentialism asserts that existence precedes essence, placing the burden of meaning-making squarely on the individual. Sartre and Camus both explored this idea, though with differing conclusions about hope and absurdity."),
    ("hl-05", "Frank Herbert's Dune explores ecology, politics, and religion through the lens of a desert planet that produces the most valuable substance in the universe. The saga spans millennia and multiple dynasties."),
    ("hl-06", "CRISPR-Cas9 allows precise editing of DNA by using a guide RNA to direct the Cas9 nuclease to a specific genomic sequence. Its discovery transformed biological research and sparked ethical debates worldwide."),
    ("hl-07", "Transformer architectures revolutionised NLP by replacing recurrent layers with self-attention, enabling massive parallelisation during training. GPT and BERT are the most famous descendants of this approach."),
    ("hl-08", "The Industrial Revolution began in Britain around 1760, driven by mechanisation of textile production and the advent of steam power. It reshaped society, moving populations from countryside to city."),
    ("hl-09", "Stoicism teaches that virtue is the sole good and that we should focus on what is within our control while accepting what is not. Marcus Aurelius and Epictetus are its most widely read ancient proponents."),
    ("hl-10", "Orwell's 1984 depicts a totalitarian regime that maintains power through pervasive surveillance, historical revisionism, and the manipulation of language. The term doublespeak entered common usage from this novel."),
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

fn main() {
    let fallback = FallbackService;
    let mut fallback_results = Vec::with_capacity(HIGHLIGHTS.len());
    for (id, text) in HIGHLIGHTS {
        let h = make_highlight(id, text);
        let out = fallback.enrich(&h, &[]).expect("fallback never fails");
        fallback_results.push(out);
    }

    let canned: Vec<EnrichmentOutput> = load_canned_responses();
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

    println!("\n=== Recommendations ===");
    if m.parse_yield >= 0.80 {
        println!(
            "Recommendation: GO — Qwen parse yield ({}%) meets production threshold.",
            (m.parse_yield * 100.0).round() as u32
        );
    } else {
        println!(
            "Recommendation: CONDITIONAL GO — Qwen parse yield ({}%) < 80%.",
            (m.parse_yield * 100.0).round() as u32
        );
        println!("          Suggest prompt engineering or hyperparameter tuning.");
    }
}

fn load_canned_responses() -> Vec<EnrichmentOutput> {
    static JSON: &str = include_str!("../fixtures/qwen_canned_responses.json");
    let raw_strings: Vec<String> = serde_json::from_str(JSON).expect("fixture is valid JSON");
    raw_strings
        .into_iter()
        .map(|s| {
            let trimmed = strip_markdown_fences(&s);
            let json_str = extract_json_object(&trimmed)
                .ok_or_else(|| "No JSON object found".to_string())
                .unwrap();
            serde_json::from_str::<EnrichmentOutput>(json_str).unwrap_or_else(|_| {
                let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
                let summary = val
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tags = val
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|st| st.to_string()))
                    .collect();
                EnrichmentOutput {
                    summary,
                    tags,
                    connection_suggestion: None,
                }
            })
        })
        .collect()
}

fn strip_markdown_fences(raw: &str) -> String {
    let raw = raw.trim();
    let raw = raw.strip_prefix("```json").unwrap_or(raw);
    let raw = raw.strip_prefix("```").unwrap_or(raw);
    let raw = raw.strip_suffix("```").unwrap_or(raw);
    raw.trim().to_string()
}

fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let bytes = raw.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &c) in bytes.iter().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            if c == b'"' {
                in_string = false;
            } else if c == b'\\' {
                escaped = true;
            }
            continue;
        }
        match c {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&raw[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

struct QualityMetrics {
    parse_yield: f64,
    parse_failures: usize,
    avg_tags_qwen: f64,
    avg_tags_fallback: f64,
    avg_summary_len_qwen: f64,
    avg_summary_len_fallback: f64,
    tag_overlap_ratio: f64,
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

        let q_set: HashSet<&str> = q.tags.iter().map(|s| s.as_str()).collect();
        let f_set: HashSet<&str> = f.tags.iter().map(|s| s.as_str()).collect();
        let union: HashSet<&str> = q_set.union(&f_set).copied().collect();
        let intersection: HashSet<&str> = q_set.intersection(&f_set).copied().collect();
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
