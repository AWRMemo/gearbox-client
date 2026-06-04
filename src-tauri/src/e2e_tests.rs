#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    use crate::ai::embedding::EmbeddingService;
    use crate::ai::fallback::FallbackService;
    use crate::ai::service::{AIService, ConnectionCandidate, Highlight};
    use crate::db::search::{search_highlights, SearchResult};
    use crate::db::set_data_dir;
    use crate::db::store::store_highlight;
    use crate::db::vector;

    static E2E_MUTEX: Mutex<()> = Mutex::new(());
    static E2E_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn setup_e2e_env() -> PathBuf {
        let _guard = vector::LANCE_DB_TEST_MUTEX.lock().unwrap();

        let dir = std::env::temp_dir().join(format!(
            "relay_e2e_test_{}",
            E2E_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);

        // Reset any stale global state
        {
            let mut conn_guard = vector::DB_CONN.lock().unwrap();
            *conn_guard = None;
        }
        {
            let mut dir_guard = crate::db::DB_DIR.lock().unwrap();
            *dir_guard = None;
        }

        set_data_dir(dir.clone());
        vector::init_vector_db(&dir).expect("init_vector_db failed");
        dir
    }

    fn load_embedding_service() -> Option<EmbeddingService> {
        let appdata =
            PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("com.gearbox.relay");
        let model_dir = appdata.join("models");
        let model = model_dir.join("all-MiniLM-L6-v2.onnx");
        let tokenizer = model_dir.join("tokenizer.json");
        if !model.exists() || !tokenizer.exists() {
            return None;
        }
        EmbeddingService::try_new(&model, &tokenizer).ok()
    }

    // Full E2E pipeline test (LanceDB + embedding + search). Slow and requires
    // model files. Run on demand or in the nightly slow-test job.
    #[ignore = "E2E pipeline test — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_e2e_capture_store_search_pipeline() {
        let _guard = E2E_MUTEX.lock().unwrap();
        let temp_dir = setup_e2e_env();

        let embedding_service = match load_embedding_service() {
            Some(svc) => svc,
            None => {
                eprintln!("Skipping e2e test: embedding model not found in app data dir");
                return;
            }
        };

        let highlight = Highlight {
            id: "e2e-hl-1".to_string(),
            text: "Transformers use self-attention to process sequences in parallel.".to_string(),
            source_url: Some("https://example.com/transformers".to_string()),
            source_title: Some("Transformer Architecture Explained".to_string()),
            source_author: None,
        };

        let vector = embedding_service
            .encode(&highlight.text)
            .expect("embedding should succeed");

        let fallback = FallbackService;
        let candidates: Vec<ConnectionCandidate> = vec![];
        let output = fallback
            .enrich(&highlight, &candidates)
            .expect("fallback enrichment should succeed");

        assert!(!output.summary.is_empty(), "Summary should not be empty");
        assert!(!output.tags.is_empty(), "Tags should not be empty");

        store_highlight(&highlight, &output, Some(&vector))
            .expect("store_highlight should succeed");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        vector::store_vector(&highlight.id, &vector, &highlight.text, now)
            .expect("store_vector should succeed");

        // Keyword-only search
        let keyword_results: Vec<SearchResult> =
            search_highlights("self-attention", None, 10).expect("search should succeed");
        assert!(
            !keyword_results.is_empty(),
            "keyword search should find the highlight"
        );
        let found = keyword_results.iter().any(|r| r.id == highlight.id);
        assert!(found, "keyword search should return the stored highlight");

        // Hybrid search
        let query_vector = embedding_service
            .encode("attention mechanisms in neural networks")
            .expect("query embedding should succeed");
        let vector_slice: &[f32] = &query_vector;
        let hybrid_results: Vec<SearchResult> =
            search_highlights("attention", Some(vector_slice), 10)
                .expect("hybrid search should succeed");
        assert!(
            !hybrid_results.is_empty(),
            "hybrid search should find results"
        );
        let hybrid_found = hybrid_results.iter().any(|r| r.id == highlight.id);
        assert!(
            hybrid_found,
            "hybrid search should return the stored highlight"
        );

        let result = hybrid_results
            .iter()
            .find(|r| r.id == highlight.id)
            .unwrap();
        assert_eq!(result.summary, output.summary);
        assert!(!result.tags.is_empty(), "Result tags should be hydrated");
        assert_eq!(result.text, highlight.text);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[ignore = "E2E pipeline test — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_e2e_multiple_highlights_isolation() {
        let _guard = E2E_MUTEX.lock().unwrap();
        let temp_dir = setup_e2e_env();

        let embedding_service = match load_embedding_service() {
            Some(svc) => svc,
            None => {
                eprintln!("Skipping e2e test: embedding model not found");
                return;
            }
        };

        let hl_a = Highlight {
            id: "hl-a".to_string(),
            text: "Rust memory safety guarantees eliminate data races at compile time.".to_string(),
            source_url: None,
            source_title: None,
            source_author: None,
        };
        let hl_b = Highlight {
            id: "hl-b".to_string(),
            text: "Python dynamic typing enables rapid prototyping but sacrifices compile-time safety."
                .to_string(),
            source_url: None,
            source_title: None,
            source_author: None,
        };

        let fallback = FallbackService;
        let candidates: Vec<ConnectionCandidate> = vec![];

        let out_a = fallback.enrich(&hl_a, &candidates).unwrap();
        let out_b = fallback.enrich(&hl_b, &candidates).unwrap();

        store_highlight(&hl_a, &out_a, None).unwrap();
        store_highlight(&hl_b, &out_b, None).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let vec_a = embedding_service.encode(&hl_a.text).unwrap();
        let vec_b = embedding_service.encode(&hl_b.text).unwrap();
        vector::store_vector(&hl_a.id, &vec_a, &hl_a.text, now).unwrap();
        vector::store_vector(&hl_b.id, &vec_b, &hl_b.text, now).unwrap();

        let q_vec = embedding_service
            .encode("Rust compiler borrow checker")
            .unwrap();
        let results = search_highlights("Rust", Some(&q_vec), 10).unwrap();
        let ids: Vec<String> = results.iter().map(|r| r.id.clone()).collect();

        assert!(
            ids.contains(&hl_a.id),
            "Search for 'Rust' should return hl_a"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
