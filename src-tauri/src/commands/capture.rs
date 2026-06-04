use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::ai::quality_monitor;
use crate::db::vector::{search_vectors, store_vector};
use relay_core::ai::fallback::{FallbackService, MAX_HIGHLIGHT_CHARS};
use relay_core::ai::service::AIService;
use relay_core::config;
use relay_core::db;
use relay_core::db::analytics;
use relay_core::telemetry;
use relay_core::types::{ConnectionCandidate, ConnectionSuggestion, EnrichmentOutput, Highlight};
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize)]
pub struct EnrichmentChunk {
    pub field: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrichResult {
    pub id: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub connection_suggestion: Option<ConnectionSuggestion>,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
}

pub(crate) fn extract_source_metadata(
    text: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let source_author = None;
    let lines: Vec<&str> = text.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
            let source_url = Some(trimmed.to_string());
            let source_title = if i > 0 {
                let prev = lines[i - 1].trim();
                if !prev.is_empty() && !prev.starts_with("http") {
                    Some(prev.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            return (source_url, source_title, source_author);
        }
    }

    (None, None, source_author)
}

/// Returns the enriched output and a boolean indicating whether the primary
/// AI service succeeded (true) or fallback was used (false).
pub(crate) fn do_enrich(
    service: Arc<dyn AIService>,
    highlight: Highlight,
    candidates: Vec<ConnectionCandidate>,
) -> (EnrichmentOutput, bool) {
    match service.enrich(&highlight, &candidates) {
        Ok(out) => (out, true),
        Err(e) => {
            tracing::warn!("AI enrichment failed, using fallback: {e}");
            let fallback = FallbackService;
            let out = fallback
                .enrich(&highlight, &candidates)
                .unwrap_or(EnrichmentOutput {
                    summary: "[enrichment failed]".to_string(),
                    tags: vec![],
                    connection_suggestion: None,
                });
            (out, false)
        }
    }
}

#[tauri::command]
pub async fn enrich_clipboard(
    app: AppHandle,
    text: String,
    on_event: Channel<EnrichmentChunk>,
    ai_service: State<'_, Arc<RwLock<Arc<dyn AIService>>>>,
) -> Result<EnrichResult, String> {
    let text = if text.len() > MAX_HIGHLIGHT_CHARS {
        let byte_end = text
            .char_indices()
            .nth(MAX_HIGHLIGHT_CHARS)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        text[..byte_end].to_string()
    } else {
        text
    };

    let (source_url, source_title, source_author) = extract_source_metadata(&text);

    let highlight = Highlight {
        id: uuid::Uuid::new_v4().to_string(),
        text: text.clone(),
        source_url,
        source_title,
        source_author,
    };

    // Semantic candidate search: encode the new highlight once, reuse embedding for storage later.
    // Read from the global that the background init thread populates.
    let embedding_vector = crate::EMBEDDING_SERVICE_GLOBAL
        .get()
        .and_then(|opt| opt.as_ref())
        .map(|es| es.encode(&highlight.text));
    let candidates = match &embedding_vector {
        Some(Ok(vector)) => match search_vectors(vector, 3) {
            Ok(results) => results
                .into_iter()
                .map(|r| ConnectionCandidate {
                    id: r.id,
                    text: r.text,
                })
                .collect(),
            Err(e) => {
                tracing::warn!("Vector search failed: {e}");
                vec![]
            }
        },
        Some(Err(e)) => {
            tracing::warn!("Embedding failed for candidate search: {e}");
            vec![]
        }
        None => vec![],
    };

    let service = {
        let guard = ai_service.read().map_err(|e| format!("Lock error: {e}"))?;
        Arc::clone(&guard)
    };
    let hl = highlight.clone();
    let cands = candidates;

    let enrich_start = Instant::now();
    let (output, parse_success) =
        tokio::task::spawn_blocking(move || do_enrich(service, hl, cands))
            .await
            .map_err(|e| format!("Inference thread panicked: {e}"))?;

    // Telemetry: enrichment latency and parse success
    telemetry::record_enrich_latency(enrich_start, parse_success);

    // Record quality metrics
    let model_name = crate::ai::model_manager::model_name();
    if let Err(e) = quality_monitor::record_quality(
        &highlight.id,
        parse_success,
        output.tags.len(),
        output.summary.len(),
        &model_name,
    ) {
        tracing::warn!("Failed to record quality metrics: {e}");
    }

    // Check for silent degradation and emit toast if threshold breached
    match quality_monitor::should_degrade_to_fallback(10, 0.7) {
        Ok(true)
            if !quality_monitor::AI_SERVICE_FAILED.load(std::sync::atomic::Ordering::SeqCst) =>
        {
            quality_monitor::AI_SERVICE_FAILED.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = app.emit("relay://ai-degraded", "AI model unstable — using fallback.");
            tracing::error!("AI quality degraded below 70% parse-success rate; forcing fallback.");
        }
        _ => {}
    }

    // Persist to SQLite
    let embedding_slice: Option<&[f32]> = embedding_vector
        .as_ref()
        .and_then(|r| r.as_ref().ok().map(|v| v.as_slice()));
    if let Err(e) = db::store::store_highlight(&highlight, &output, embedding_slice) {
        tracing::warn!("Failed to persist highlight: {e}");
    }

    // Store vector in LanceDB, reusing the pre-computed embedding from candidate search
    if let Some(Ok(ref vector)) = embedding_vector {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if let Err(e) = store_vector(&highlight.id, vector, &highlight.text, now) {
            tracing::warn!("Failed to store vector: {e}");
        }
    } else if let Some(Err(ref e)) = embedding_vector {
        tracing::warn!("Failed to encode embedding for storage: {e}");
    }

    if let Ok(user_id) = config::get_device_id() {
        if let Ok(true) = config::is_first_highlight_capture() {
            let _ =
                analytics::log_event("first_highlight_captured", None, Some(user_id), None, None);
        }
    }

    for ch in output.summary.chars() {
        let msg = EnrichmentChunk {
            field: String::from("summary"),
            value: Some(ch.to_string()),
        };
        if on_event.send(msg).is_err() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    }

    for tag in &output.tags {
        let msg = EnrichmentChunk {
            field: String::from("tag"),
            value: Some(tag.clone()),
        };
        if on_event.send(msg).is_err() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let connection_value = output
        .connection_suggestion
        .as_ref()
        .map(|cs| cs.bridging_sentence.clone());
    let msg = EnrichmentChunk {
        field: "connection".into(),
        value: connection_value,
    };
    if on_event.send(msg).is_err() {
        return Ok(EnrichResult {
            id: highlight.id.clone(),
            summary: output.summary,
            tags: output.tags,
            connection_suggestion: output.connection_suggestion,
            source_url: highlight.source_url.clone(),
            source_title: highlight.source_title.clone(),
            source_author: highlight.source_author.clone(),
        });
    }

    let msg = EnrichmentChunk {
        field: "done".into(),
        value: None,
    };
    if on_event.send(msg).is_err() {
        return Ok(EnrichResult {
            id: highlight.id.clone(),
            summary: output.summary,
            tags: output.tags,
            connection_suggestion: output.connection_suggestion,
            source_url: highlight.source_url.clone(),
            source_title: highlight.source_title.clone(),
            source_author: highlight.source_author.clone(),
        });
    }

    Ok(EnrichResult {
        id: highlight.id,
        summary: output.summary,
        tags: output.tags,
        connection_suggestion: output.connection_suggestion,
        source_url: highlight.source_url,
        source_title: highlight.source_title,
        source_author: highlight.source_author,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::service::{
        AIService, ConnectionCandidate, ConnectionSuggestion, EnrichmentOutput, Highlight,
    };

    struct MockAIService {
        output: EnrichmentOutput,
    }

    impl AIService for MockAIService {
        fn enrich(
            &self,
            _highlight: &Highlight,
            _candidates: &[ConnectionCandidate],
        ) -> Result<EnrichmentOutput, String> {
            Ok(self.output.clone())
        }
    }

    #[test]
    fn test_do_enrich_returns_service_output() {
        let expected = EnrichmentOutput {
            summary: "Mock summary.".into(),
            tags: vec!["mock".into(), "ai".into()],
            connection_suggestion: Some(ConnectionSuggestion {
                source_highlight_id: "prev-1".into(),
                bridging_sentence: "Links nicely.".into(),
            }),
        };
        let service: Arc<dyn AIService> = Arc::new(MockAIService {
            output: expected.clone(),
        });
        let highlight = Highlight {
            id: "hl-1".into(),
            text: "Some text.".into(),
            source_url: None,
            source_title: None,
            source_author: None,
        };
        let (result, parse_success) = do_enrich(service, highlight, vec![]);
        assert_eq!(result.summary, expected.summary);
        assert_eq!(result.tags, expected.tags);
        assert!(result.connection_suggestion.is_some());
        assert!(parse_success);
        assert_eq!(
            result.connection_suggestion.unwrap().bridging_sentence,
            "Links nicely."
        );
    }

    #[test]
    fn test_do_enrich_fallback_on_error() {
        struct FailingService;

        impl AIService for FailingService {
            fn enrich(
                &self,
                _highlight: &Highlight,
                _candidates: &[ConnectionCandidate],
            ) -> Result<EnrichmentOutput, String> {
                Err("model load failed".into())
            }
        }

        let service: Arc<dyn AIService> = Arc::new(FailingService);
        let highlight = Highlight {
            id: "hl-2".into(),
            text: "Fallback should work here.".into(),
            source_url: None,
            source_title: None,
            source_author: None,
        };
        let result = do_enrich(service, highlight, vec![]);
        assert!(!result.0.summary.is_empty());
        assert!(
            result.0.summary != "[enrichment failed]",
            "fallback should produce deterministic summary, got: {}",
            result.0.summary
        );
        assert!(result.0.connection_suggestion.is_none());
    }

    #[test]
    fn test_extract_source_metadata_with_url() {
        let text = "Page Title\nhttps://example.com/article\nSome body text.";
        let (url, title, author) = extract_source_metadata(text);
        assert_eq!(url, Some("https://example.com/article".into()));
        assert_eq!(title, Some("Page Title".into()));
        assert!(author.is_none());
    }

    #[test]
    fn test_extract_source_metadata_no_url() {
        let text = "Just some plain text without any URL.";
        let (url, title, author) = extract_source_metadata(text);
        assert!(url.is_none());
        assert!(title.is_none());
        assert!(author.is_none());
    }

    #[test]
    fn test_extract_source_metadata_url_at_start() {
        let text = "https://example.com/article\nBody text.";
        let (url, title, author) = extract_source_metadata(text);
        assert_eq!(url, Some("https://example.com/article".into()));
        assert!(title.is_none());
        assert!(author.is_none());
    }
}
