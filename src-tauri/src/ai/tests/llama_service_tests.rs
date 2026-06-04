use crate::ai::llama_service::LlamaService;
use relay_core::types::EnrichmentOutput;

#[allow(dead_code)]
fn assert_eq_enrichment(out: &EnrichmentOutput, expected_summary: &str, expected_tags: &[&str]) {
    assert_eq!(out.summary, expected_summary);
    let expected: Vec<String> = expected_tags.iter().map(|s| s.to_string()).collect();
    assert_eq!(out.tags, expected);
}

// ==========================
// Well-formed Qwen 3.5 outputs (simulated)
// ==========================

#[test]
fn wf_01_basic() {
    let raw =
        r#"{"summary": "Rust is fast.", "tags": ["rust", "performance"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Rust is fast.");
    assert_eq!(out.tags, vec!["rust", "performance"]);
    assert!(out.connection_suggestion.is_none());
}

#[test]
fn wf_02_with_connection() {
    let raw = r#"{"summary": "LanceDB enables local vector search.", "tags": ["lancedb", "vector-db"], "connection": {"source_highlight_id": "abc-123", "bridging_sentence": "Both use Rust for performance."}}"#;
    let out = LlamaService::parse_output(raw, "abc-123").unwrap();
    assert_eq!(&out.summary, "LanceDB enables local vector search.");
    assert_eq!(out.tags, vec!["lancedb", "vector-db"]);
    assert_eq!(
        out.connection_suggestion
            .as_ref()
            .unwrap()
            .bridging_sentence,
        "Both use Rust for performance."
    );
}

#[test]
fn wf_03_fenced() {
    let raw =
        "```json\n{\"summary\": \"Fenced.\", \"tags\": [\"fence\"], \"connection\": null}\n```";
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Fenced.");
    assert_eq!(out.tags, vec!["fence"]);
}

#[test]
fn wf_04_extra_whitespace() {
    let raw = r#"   {"summary": "Whitespace.", "tags": ["ws"], "connection": null}   "#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Whitespace.");
}

#[test]
fn wf_05_plain_string_connection() {
    let raw = r#"{"summary": "Summary.", "tags": ["a"], "connection": "Plain connection."}"#;
    let out = LlamaService::parse_output(raw, "prior-hl-42").unwrap();
    assert_eq!(
        out.connection_suggestion
            .as_ref()
            .unwrap()
            .bridging_sentence,
        "Plain connection."
    );
    assert_eq!(
        out.connection_suggestion
            .as_ref()
            .unwrap()
            .source_highlight_id,
        "prior-hl-42"
    );
}

#[test]
fn wf_06_long_tags() {
    let raw = r#"{"summary": "Quantum computing uses superposition.", "tags": ["quantum-computing", "superposition", "entanglement", "qubits", "algorithms"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(out.tags.len(), 5);
    assert!(out.tags.contains(&"qubits".to_string()));
}

#[test]
fn wf_07_unicode_summary() {
    let raw = r#"{"summary": "Rust is great for low-level coding and has zero-cost abstractions with fearless concurrency.", "tags": ["rust", "low-level", "concurrency"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert!(out.summary.contains("zero-cost"));
    assert_eq!(out.tags.len(), 3);
}

#[test]
fn wf_08_empty_tags_array() {
    let raw = r#"{"summary": "No tags.", "tags": [], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(out.tags, Vec::<String>::new());
    assert_eq!(&out.summary, "No tags.");
}

#[test]
fn wf_09_single_tag() {
    let raw = r#"{"summary": "Solo.", "tags": ["solo"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(out.tags, vec!["solo"]);
}

#[test]
fn wf_10_extra_fields() {
    let raw = r#"{"summary": "Extra.", "tags": ["x"], "connection": null, "confidence": 0.95}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Extra.");
}

#[test]
fn wf_11_trailing_text() {
    let raw =
        r#"{"summary": "Trailing.", "tags": ["trail"], "connection": null} some extra text after"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Trailing.");
}

#[test]
fn wf_12_deeply_nested() {
    let raw = r#"{"summary": "Deep.", "tags": ["deep"], "connection": {"source_highlight_id": "x", "bridging_sentence": "Very deep."}}"#;
    let out = LlamaService::parse_output(raw, "x").unwrap();
    assert_eq!(
        out.connection_suggestion
            .as_ref()
            .unwrap()
            .bridging_sentence,
        "Very deep."
    );
}

#[test]
fn wf_13_null_summary_and_nonempty_tags() {
    let raw = r#"{"summary": null, "tags": ["null-summary"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "");
    assert_eq!(out.tags, vec!["null-summary"]);
}

#[test]
fn wf_14_special_chars_in_tags() {
    let raw = r#"{"summary": "Special.", "tags": ["c++", "c#", "f#"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(out.tags, vec!["c++", "c#", "f#"]);
}

#[test]
fn wf_15_connection_with_source_id() {
    let raw = r#"{"summary": "Summary.", "tags": ["a"], "connection": {"source_highlight_id": "abc-123", "bridging_sentence": "Bridging sentence."}}"#;
    let out = LlamaService::parse_output(raw, "abc-123").unwrap();
    assert_eq!(
        out.connection_suggestion
            .as_ref()
            .unwrap()
            .source_highlight_id,
        "abc-123"
    );
}

#[test]
fn wf_16_leading_newlines() {
    let raw = r#"

{"summary": "Start late.", "tags": ["late"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Start late.");
}

#[test]
fn wf_17_leading_garbage() {
    let raw =
        "OK here is the json\n{\"summary\": \"Garbage pref.\", \"tags\": [\"pref\"], \"connection\": null}";
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Garbage pref.");
}

#[test]
fn wf_18_fenced_no_lang() {
    let raw =
        "```\n{\"summary\": \"No lang fence.\", \"tags\": [\"nolang\"], \"connection\": null}\n```";
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "No lang fence.");
}

#[test]
fn wf_19_large_tag_count() {
    let raw = r#"{"summary": "Many tags.", "tags": ["a","b","c","d","e","f","g","h","i","j"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(out.tags.len(), 10);
}

#[test]
fn wf_20_strict_deser() {
    let raw = r#"{"summary": "Strict.", "tags": ["strict"], "connection": null}"#;
    let out = LlamaService::parse_output(raw, "").unwrap();
    assert_eq!(&out.summary, "Strict.");
    assert_eq!(out.tags, vec!["strict"]);
    assert!(out.connection_suggestion.is_none());
}

// ==========================
// Malformed outputs
// ==========================

#[test]
fn mf_01_no_json() {
    let raw = "This is not JSON at all.";
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_02_empty() {
    let raw = "";
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_03_unclosed_brace() {
    let raw = r#"{"summary": "Unclosed", "tags": ["a"], "connection": null"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_04_missing_summary() {
    let raw = r#"{"tags": ["only"], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_05_missing_tags() {
    let raw = r#"{"summary": "Only summary.", "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_06_summary_not_string() {
    let raw = r#"{"summary": 42, "tags": ["a"], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_07_tags_not_array() {
    let raw = r#"{"summary": "Bad tags.", "tags": "not-array", "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_08_tags_with_non_strings() {
    let raw = r#"{"summary": "Mixed tags.", "tags": ["ok", 42, true], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    // filter_map skips non-strings, leaving only "ok"
    assert!(result.is_ok());
    assert_eq!(result.unwrap().tags, vec!["ok"]);
}

#[test]
fn mf_09_both_empty() {
    let raw = r#"{"summary": "", "tags": [], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_10_trailing_comma() {
    let raw = r#"{"summary": "Comma.", "tags": ["a",], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_11_null_outer() {
    let raw = "null";
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_12_array_outer() {
    let raw = r#"[{"summary": "Array.", "tags": ["a"], "connection": null}]"#;
    let result = LlamaService::parse_output(raw, "");
    // Defensive parser extracts the first JSON object inside the array
    assert!(result.is_ok());
    assert_eq!(result.unwrap().summary, "Array.");
}

#[test]
fn mf_13_invalid_json() {
    let raw = r#"{"summary": "Bad\x"}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_14_connection_invalid() {
    let raw = r#"{"summary": "Summary.", "tags": ["a"], "connection": 42}"#;
    let result = LlamaService::parse_output(raw, "");
    // Defensive parser ignores non-object/string connection values
    assert!(result.is_ok());
    assert!(result.unwrap().connection_suggestion.is_none());
}

#[test]
fn mf_15_unescaped_quotes() {
    let raw = r#"{"summary": "He said "hello".", "tags": ["quote"], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_16_missing_brace_open() {
    let raw = r#""summary": "Missing open", "tags": ["a"], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_17_fenced_but_broken_inside() {
    let raw = "```json\n{\"summary\": \"Broken\", \"tags\": [\"b\"]\n```";
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}

#[test]
fn mf_18_incomplete_connection() {
    let raw =
        r#"{"summary": "Summary.", "tags": ["a"], "connection": {"source_highlight_id": "x"}}"#;
    let result = LlamaService::parse_output(raw, "");
    // Defensive parser treats incomplete connection as absent
    assert!(result.is_ok());
    assert!(result.unwrap().connection_suggestion.is_none());
}

#[test]
fn mf_19_xml_wrapping() {
    let raw = "<json>{\"summary\": \"XML.\", \"tags\": [\"xml\"], \"connection\": null}</json>";
    let result = LlamaService::parse_output(raw, "");
    // Defensive parser extracts JSON object from XML wrapper
    assert!(result.is_ok());
    assert_eq!(result.unwrap().summary, "XML.");
}

#[test]
fn mf_20_nested_json_strings() {
    let raw = r#"{"summary": "{"nested": true}", "tags": ["nested"], "connection": null}"#;
    let result = LlamaService::parse_output(raw, "");
    assert!(result.is_err());
}
