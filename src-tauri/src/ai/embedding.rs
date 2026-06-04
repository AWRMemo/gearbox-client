use std::path::Path;
use std::sync::{Arc, Mutex};

use ndarray::Array2;
use ort::session::Session;
use ort::value::Value;
use tokenizers::Tokenizer;

const MAX_SEQ_LENGTH: usize = 512;

struct Inner {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
}

/// Lightweight, thread-safe wrapper around an ONNX embedding model.
///
/// The model and tokenizer are loaded once at construction time.
/// Cloning the service is cheap because it uses an `Arc` internally.
#[derive(Clone)]
pub struct EmbeddingService {
    inner: Arc<Inner>,
}

impl EmbeddingService {
    /// Load the ONNX model and its tokenizer from disk.
    ///
    /// # Errors
    /// Returns `Err` if the model file cannot be loaded, the tokenizer cannot be
    /// parsed, or the ONNX runtime fails to initialise the session.
    pub fn try_new(model_path: &Path, tokenizer_path: &Path) -> Result<Self, String> {
        let session = Session::builder()
            .map_err(|e| format!("Failed to create ONNX session builder: {e}"))?
            .commit_from_file(model_path)
            .map_err(|e| format!("Failed to load ONNX model: {e}"))?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {e}"))?;

        Ok(Self {
            inner: Arc::new(Inner {
                session: Mutex::new(session),
                tokenizer,
            }),
        })
    }

    /// Encode a single text passage into a normalised embedding vector.
    ///
    /// The input is tokenised, truncated to `MAX_SEQ_LENGTH` tokens, run through
    /// the ONNX model, mean-pooled with the attention mask, and L2-normalised.
    ///
    /// # Errors
    /// Returns `Err` for empty input, tokenisation failures, or ONNX runtime errors.
    pub fn encode(&self, text: &str) -> Result<Vec<f32>, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("Input text is empty".to_string());
        }

        let mut tokenizer = self.inner.tokenizer.clone();
        let truncation = tokenizers::TruncationParams {
            max_length: MAX_SEQ_LENGTH,
            ..Default::default()
        };
        tokenizer
            .with_truncation(Some(truncation))
            .map_err(|e| format!("Failed to configure truncation: {e}"))?;

        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenisation failed: {e}"))?;

        let ids = encoding.get_ids();
        let mask = encoding.get_attention_mask();
        let seq_len = ids.len();

        let input_ids =
            Array2::from_shape_vec((1, seq_len), ids.iter().map(|&x| x as i64).collect())
                .map_err(|e| format!("Failed to build input_ids tensor: {e}"))?;

        let attention_mask =
            Array2::from_shape_vec((1, seq_len), mask.iter().map(|&x| x as i64).collect())
                .map_err(|e| format!("Failed to build attention_mask tensor: {e}"))?;

        // token_type_ids are all zeros for single-sentence (Sentence-BERT style) input
        let token_type_ids = Array2::from_shape_vec((1, seq_len), vec![0i64; seq_len])
            .map_err(|e| format!("Failed to build token_type_ids tensor: {e}"))?;

        let input_ids_value =
            Value::from_array(input_ids).map_err(|e| format!("input_ids Value error: {e}"))?;
        let attention_mask_value = Value::from_array(attention_mask)
            .map_err(|e| format!("attention_mask Value error: {e}"))?;
        let token_type_ids_value = Value::from_array(token_type_ids)
            .map_err(|e| format!("token_type_ids Value error: {e}"))?;

        let inputs: Vec<(std::borrow::Cow<'static, str>, Value)> = vec![
            ("input_ids".into(), input_ids_value.into()),
            ("attention_mask".into(), attention_mask_value.into()),
            ("token_type_ids".into(), token_type_ids_value.into()),
        ];

        let mut binding = self
            .inner
            .session
            .lock()
            .map_err(|e| format!("Session lock error: {e}"))?;
        let outputs = binding
            .run(inputs)
            .map_err(|e| format!("ONNX inference failed: {e}"))?;

        let (shape, data) = outputs
            .get("last_hidden_state")
            .ok_or("ONNX output 'last_hidden_state' not found")?
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract output tensor: {e}"))?;

        if shape.len() != 3 {
            return Err(format!(
                "Expected 3-D output tensor [batch, seq, hidden], got shape {:?}",
                shape
            ));
        }
        let hidden_size = shape[2] as usize;

        let mut embedding = vec![0.0_f32; hidden_size];
        let mut mask_sum = 0.0_f32;
        for (token_idx, m) in mask.iter().enumerate() {
            let weight = *m as f32;
            mask_sum += weight;
            let base = token_idx * hidden_size;
            for dim in 0..hidden_size {
                embedding[dim] += data[base + dim] * weight;
            }
        }

        if mask_sum > 0.0 {
            for val in embedding.iter_mut().take(hidden_size) {
                *val /= mask_sum;
            }
        }

        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }

        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::embedding_model_manager;
    use std::sync::{Arc, OnceLock};
    use std::thread;

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    fn get_test_service() -> Result<Arc<EmbeddingService>, String> {
        static SERVICE: OnceLock<Result<Arc<EmbeddingService>, String>> = OnceLock::new();
        SERVICE
            .get_or_init(|| {
                let dir = std::env::temp_dir().join("relay_embedding_test");
                std::fs::create_dir_all(&dir).map_err(|e| format!("{e}"))?;
                let (model, tokenizer) = embedding_model_manager::ensure_embedding_model(&dir)?;
                Ok(Arc::new(EmbeddingService::try_new(&model, &tokenizer)?))
            })
            .clone()
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_simple_sentence() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let embedding = service
            .encode("The quick brown fox jumps over the lazy dog.")
            .unwrap();
        assert!(!embedding.is_empty(), "embedding must not be empty");

        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_empty_string_returns_error() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let result = service.encode("");
        assert!(result.is_err(), "empty string must return an error");
        let err = result.unwrap_err().to_lowercase();
        assert!(
            err.contains("empty"),
            "error message should mention 'empty', got: {err}"
        );

        let result_ws = service.encode("   \n\t  ");
        assert!(
            result_ws.is_err(),
            "whitespace-only string must return an error"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_similar_sentences_high_cosine() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let a = service.encode("A happy dog plays in the park.").unwrap();
        let b = service
            .encode("A joyful puppy runs in the garden.")
            .unwrap();
        let sim = cosine_similarity(&a, &b);
        // all-MiniLM-L6-v2 on CPU produces moderate similarity for these semantically similar sentences
        assert!(
            sim > 0.5,
            "similar sentences should have cosine similarity > 0.5, got {sim}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_unrelated_sentences_low_cosine() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let a = service
            .encode("The stock market crashed yesterday.")
            .unwrap();
        let b = service
            .encode("A fluffy cat sleeps on the windowsill.")
            .unwrap();
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim < 0.5,
            "unrelated sentences should have cosine similarity < 0.5, got {sim}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_unicode() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        // CJK + emoji
        let emb = service.encode("日本語のテキストと絵文字 🚀🎉").unwrap();
        assert!(
            !emb.is_empty(),
            "unicode text must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "unicode embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_single_word() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service.encode("hello").unwrap();
        assert!(
            !emb.is_empty(),
            "single word must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "single-word embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_repeated_text_consistent() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let text = "Consistency is the key to reliable embeddings.";
        let a = service.encode(text).unwrap();
        let b = service.encode(text).unwrap();
        assert_eq!(
            a.len(),
            b.len(),
            "repeated encodings must have same dimension"
        );
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim > 0.9999,
            "identical text should yield cosine similarity ~1.0, got {sim}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_different_texts_different() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let a = service.encode("Machine learning is fascinating.").unwrap();
        let b = service.encode("The weather today is quite rainy.").unwrap();
        // They should not be exactly equal vectors
        let all_equal = a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-6);
        assert!(
            !all_equal,
            "different texts should produce different embedding vectors"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_long_text_truncation() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        // build text that will definitely exceed 512 tokens (~1 token/word)
        let words: Vec<String> = (0..800).map(|i| format!("word{i}")).collect();
        let long_text = words.join(" ");
        let emb = service.encode(&long_text).unwrap();
        assert!(!emb.is_empty(), "long text (>512 tokens) must still encode");
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "truncated long-text embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_very_long_input() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let text = "x".repeat(11_000);
        let emb = service.encode(&text).unwrap();
        assert!(!emb.is_empty(), ">10k char input must still encode");
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "very-long input embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_special_chars_only() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service.encode("!@#$%^&*()_+-=[]{}|;':\",./<>?").unwrap();
        assert!(
            !emb.is_empty(),
            "special-char-only text must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "special-char embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_newlines_tabs() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service
            .encode("Line one\nLine two\tindented\n\nFinal line.")
            .unwrap();
        assert!(
            !emb.is_empty(),
            "text with newlines and tabs must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "newline/tab embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_numeric_only() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service.encode("1234567890 3.14159 2.71828").unwrap();
        assert!(
            !emb.is_empty(),
            "numeric-only text must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "numeric-only embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_whitespace_only_returns_error() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let result = service.encode("     ");
        assert!(
            result.is_err(),
            "whitespace-only input must return an error"
        );
        let err = result.unwrap_err().to_lowercase();
        assert!(
            err.contains("empty"),
            "error message should mention 'empty', got: {err}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_single_char() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service.encode("a").unwrap();
        assert!(
            !emb.is_empty(),
            "single character must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "single-char embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_exact_same_text_identity() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let a = service.encode("The cat sat on the mat.").unwrap();
        let b = service.encode("The cat sat on the mat.").unwrap();
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim > 0.9999,
            "exact same text should yield cosine similarity ~1.0, got {sim}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_different_languages() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let french = service.encode("Le chat dort sur le canapé.").unwrap();
        let german = service.encode("Die Katze schläft auf dem Sofa.").unwrap();
        assert_eq!(
            french.len(),
            german.len(),
            "embeddings must share same dimension"
        );
        let sim = cosine_similarity(&french, &german);
        // Semantically equivalent sentences in different languages should be moderately similar
        assert!(
            sim > 0.4,
            "equivalent sentences in different languages should have cosine similarity > 0.4, got {sim}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_case_insensitive_similarity() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let lower = service.encode("the quick brown fox").unwrap();
        let upper = service.encode("THE QUICK BROWN FOX").unwrap();
        let mixed = service.encode("ThE qUiCk BrOwN fOx").unwrap();
        let sim_low_up = cosine_similarity(&lower, &upper);
        let sim_low_mix = cosine_similarity(&lower, &mixed);
        assert!(
            sim_low_up > 0.95,
            "case changes should preserve high similarity, got {sim_low_up}"
        );
        assert!(
            sim_low_mix > 0.95,
            "mixed case should preserve high similarity, got {sim_low_mix}"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_punctuation_only() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service.encode("...,,,!!!???;::").unwrap();
        assert!(
            !emb.is_empty(),
            "punctuation-only text must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "punctuation-only embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_encode_mixed_alphanumeric() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let emb = service.encode("abc123 XYZ789 !@#").unwrap();
        assert!(
            !emb.is_empty(),
            "mixed alphanumeric text must produce a non-empty embedding"
        );
        let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "mixed-alphanumeric embedding should be L2-normalised (norm = {norm})"
        );
    }

    #[ignore = "Embedding model download + ONNX inference — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_session_concurrent_access() {
        let service = get_test_service().expect("failed to initialise test embedding service");
        let service_clone = Arc::clone(&service);

        let mut handles = vec![];
        for i in 0..4 {
            let svc = Arc::clone(&service_clone);
            handles.push(thread::spawn(move || {
                let text = format!("Thread {i} is encoding this sentence concurrently.");
                svc.encode(&text)
            }));
        }

        for (i, h) in handles.into_iter().enumerate() {
            let emb = h
                .join()
                .expect("thread panicked")
                .expect("concurrent encode failed");
            assert!(
                !emb.is_empty(),
                "thread {i} must produce a non-empty embedding"
            );
            let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!(
                (norm - 1.0).abs() < 1e-4,
                "thread {i} embedding should be L2-normalised (norm = {norm})"
            );
        }
    }
}
