use crate::ai::service::{EnrichmentOutput, ConnectionSuggestion};

#[test]
fn test_parse_real_model_output() {
    let raw = r#"  {"summary": "Bitcoin rose to $82,000 as traders welcomed regulatory progress in Washington. The largest crypto later retreated to $81,500, still up 2.5% over the past 24 hours.", "tags": ["bitcoin", "regulatory", "crypto"], "connection": null}

 thinking
Thinking Process:

1.  **Analyze the Request:**"#;
    let result = serde_json::from_str::<EnrichmentOutput>(raw.trim().lines().next().unwrap());
    eprintln!("{:?}", result);
    assert!(result.is_ok());
}
