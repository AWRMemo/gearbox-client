use crate::ai::error::AiError;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;
use relay_core::ai::fallback::FallbackService;
use relay_core::ai::service::{
    AIService, ConnectionCandidate, ConnectionSuggestion, EnrichmentOutput, Highlight,
};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

const SYSTEM_PROMPT: &str = "You are a personal knowledge assistant. Respond ONLY with valid JSON.";

struct Inner {
    backend: LlamaBackend,
    model: LlamaModel,
}

#[derive(Clone)]
pub struct LlamaService {
    inner: Arc<std::sync::Mutex<Inner>>,
}

impl LlamaService {
    pub fn try_load(model_path: &Path) -> Result<Arc<dyn AIService>, String> {
        let backend = LlamaBackend::init().map_err(|e| format!("Failed to init backend: {e}"))?;
        let model = LlamaModel::load_from_file(&backend, model_path, &LlamaModelParams::default())
            .map_err(|e| format!("Failed to load model: {e}"))?;
        Ok(Arc::new(Self {
            inner: Arc::new(std::sync::Mutex::new(Inner { backend, model })),
        }))
    }

    /// Async inference entry-point for a raw text string.
    pub async fn enrich_text(&self, text: &str) -> Result<EnrichmentOutput, AiError> {
        let highlight = Highlight {
            id: uuid::Uuid::new_v4().to_string(),
            text: text.to_string(),
            source_url: None,
            source_title: None,
            source_author: None,
        };
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.enrich_sync(&highlight, &[]))
            .await
            .map_err(|e| AiError::Inference(format!("Thread panicked: {e}")))?
    }

    fn enrich_sync(
        &self,
        highlight: &Highlight,
        candidates: &[ConnectionCandidate],
    ) -> Result<EnrichmentOutput, AiError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| AiError::Inference(format!("Lock error: {e}")))?;
        Self::enrich_inner(&mut inner, highlight, candidates)
    }

    fn build_user_content(highlight: &Highlight, candidates: &[ConnectionCandidate]) -> String {
        let has_candidate = candidates
            .first()
            .map(|c| !c.text.is_empty())
            .unwrap_or(false);
        let candidate_text = if has_candidate {
            format!(
                "\n\nCandidate (id: {}): \"{}\"",
                candidates[0].id, candidates[0].text
            )
        } else {
            String::new()
        };
        format!(
            "Extract a JSON object from the highlighted passage below.\n\
            The JSON must contain exactly \"tags\" (array of strings) and \"summary\" (string).\n\
            Optionally include \"connection\" (object with source_highlight_id and bridging_sentence, or null).\n\n\
            Text: \"{}\"{}\nJSON: ",
            highlight.text, candidate_text
        )
    }

    fn apply_chat_template(inner: &Inner, user_content: &str) -> Result<String, AiError> {
        let tmpl = inner
            .model
            .chat_template(None)
            .map_err(|e| AiError::ModelLoad(format!("Missing chat template: {e}")))?;
        let system = LlamaChatMessage::new("system".into(), SYSTEM_PROMPT.into())
            .map_err(|e| AiError::Inference(format!("System prompt invalid: {e}")))?;
        let user = LlamaChatMessage::new("user".into(), user_content.into())
            .map_err(|e| AiError::Inference(format!("User prompt invalid: {e}")))?;
        inner
            .model
            .apply_chat_template(&tmpl, &[system, user], true)
            .map_err(|e| AiError::Inference(format!("Template apply failed: {e}")))
    }

    fn enrich_inner(
        inner: &mut Inner,
        highlight: &Highlight,
        candidates: &[ConnectionCandidate],
    ) -> Result<EnrichmentOutput, AiError> {
        let user_content = Self::build_user_content(highlight, candidates);
        let prompt = match Self::apply_chat_template(inner, &user_content) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Chat template failed ({e}), falling back to plain prompt.");
                format!("{SYSTEM_PROMPT}\n\n{user_content}")
            }
        };

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(4096))
            .with_n_batch(2048);
        let mut context = inner
            .model
            .new_context(&inner.backend, ctx_params)
            .map_err(|e| AiError::Inference(format!("Failed to create context: {e}")))?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
            LlamaSampler::greedy(),
        ]);

        let tokens = inner
            .model
            .str_to_token(&prompt, AddBos::Never)
            .map_err(|e| AiError::Inference(format!("Tokenization failed: {e}")))?;
        let n_ctx = context.n_ctx() as usize;
        if tokens.len() >= n_ctx {
            return Err(AiError::PromptTooLong {
                tokens: tokens.len(),
                limit: n_ctx,
            });
        }
        let max_new_tokens = 1024;
        let mut output_tokens = Vec::new();
        let mut batch = LlamaBatch::new(n_ctx, 1);
        batch
            .add_sequence(&tokens, 0, true)
            .map_err(|e| AiError::Inference(format!("Failed to add prompt batch: {e}")))?;
        context
            .decode(&mut batch)
            .map_err(|e| AiError::Inference(format!("Decode failed: {e}")))?;
        let eos_token = inner.model.token_eos();
        let mut pos = tokens.len() as i32;
        for _ in 0..max_new_tokens {
            let token = sampler.sample(&context, -1);
            output_tokens.push(token);
            if token == eos_token || inner.model.is_eog_token(token) {
                break;
            }
            let mut new_batch = LlamaBatch::new(1, 1);
            new_batch
                .add(token, pos, &[0], true)
                .map_err(|e| AiError::Inference(format!("Failed to add token: {e}")))?;
            context
                .decode(&mut new_batch)
                .map_err(|e| AiError::Inference(format!("Decode failed: {e}")))?;
            pos += 1;
        }
        let raw_output = Self::tokens_to_string(&inner.model, &output_tokens)?;
        let candidate_id = candidates.first().map(|c| c.id.as_str()).unwrap_or("");
        match Self::parse_output(&raw_output, candidate_id) {
            Ok(out) => Ok(out),
            Err(e) => {
                eprintln!("LLM parse failed ({e}), falling back to deterministic extraction.");
                FallbackService
                    .enrich(highlight, candidates)
                    .map_err(|e| AiError::Fallback(format!("Fallback also failed: {e}")))
            }
        }
    }

    fn tokens_to_string(model: &LlamaModel, tokens: &[LlamaToken]) -> Result<String, AiError> {
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut result = String::new();
        for token in tokens {
            let piece = model
                .token_to_piece(*token, &mut decoder, false, None)
                .map_err(|e| AiError::Parse(format!("Failed to decode token: {e}")))?;
            result.push_str(&piece);
        }
        Ok(result)
    }

    // ==========================
    // Multi-layer defensive parser
    // ==========================

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

    fn parse_output(raw: &str, candidate_id: &str) -> Result<EnrichmentOutput, AiError> {
        let cleaned = Self::strip_markdown_fences(raw);
        let json_str = Self::extract_json_object(&cleaned)
            .ok_or_else(|| AiError::Parse("No JSON object found in output".into()))?;

        // Strict deserialization first
        if let Ok(output) = serde_json::from_str::<EnrichmentOutput>(json_str) {
            // Reject degenerate outputs even if structurally valid
            if !output.summary.is_empty() || !output.tags.is_empty() {
                return Ok(output);
            }
        }

        // Loose Value parsing
        let val: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| AiError::Json(format!("Invalid JSON: {e}")))?;

        let summary = val
            .get("summary")
            .and_then(|v| {
                if v.is_null() {
                    Some("")
                } else {
                    v.as_str()
                }
            })
            .ok_or_else(|| AiError::Parse("Missing 'summary' field".into()))?
            .to_string();

        let tags = val
            .get("tags")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AiError::Parse("Missing 'tags' field".into()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        if summary.is_empty() && tags.is_empty() {
            return Err(AiError::Parse("Both summary and tags are empty".into()));
        }

        let connection = val.get("connection").and_then(|v| {
            if v.is_null() {
                None
            } else if let Some(s) = v.as_str() {
                Some(ConnectionSuggestion {
                    source_highlight_id: candidate_id.to_string(),
                    bridging_sentence: s.to_string(),
                })
            } else {
                v.get("bridging_sentence")
                    .and_then(|bs| bs.as_str())
                    .map(|bs| ConnectionSuggestion {
                        source_highlight_id: v
                            .get("source_highlight_id")
                            .and_then(|x| x.as_str())
                            .unwrap_or(candidate_id)
                            .to_string(),
                        bridging_sentence: bs.to_string(),
                    })
            }
        });

        Ok(EnrichmentOutput {
            summary,
            tags,
            connection_suggestion: connection,
        })
    }
}

impl AIService for LlamaService {
    fn enrich(
        &self,
        highlight: &Highlight,
        candidates: &[ConnectionCandidate],
    ) -> Result<EnrichmentOutput, String> {
        self.enrich_sync(highlight, candidates)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
#[path = "tests/llama_service_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/ai_quality_bench.rs"]
mod ai_quality_bench;
