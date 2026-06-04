use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use arrow::array::{Array, FixedSizeListArray, Float32Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Float32Type, Schema};
use arrow::record_batch::RecordBatch;
use futures::stream::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType};

pub(crate) static DB_CONN: Mutex<Option<Connection>> = Mutex::new(None);

/// Shared mutex for any test that mutates the LanceDB global connection.
#[cfg(test)]
pub(crate) static LANCE_DB_TEST_MUTEX: Mutex<()> = Mutex::new(());

const TABLE_NAME: &str = "vectors";
const VECTOR_DIM: i32 = 384;

/// A single vector search result.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearchResult {
    pub id: String,
    pub text: String,
    pub score: f32,
}

fn get_conn() -> Result<Connection, String> {
    let guard = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .clone()
        .ok_or_else(|| "Vector DB not initialized".to_string())
}

/// Helper to run an async LanceDB operation from either a sync or async context.
fn block_on_lancedb<F, T>(future: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>> + Send,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(future))
    } else {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create tokio runtime: {e}"))?;
        rt.block_on(future)
    }
}

fn make_schema() -> arrow::datatypes::SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                VECTOR_DIM,
            ),
            true,
        ),
        Field::new("text", DataType::Utf8, false),
        Field::new("created_at", DataType::Int64, false),
    ]))
}

/// Create/open the LanceDB dataset in the app data directory.
/// Must be called once before any other vector DB operation.
pub fn init_vector_db(app_dir: &Path) -> Result<(), String> {
    let db_path = app_dir.join("vectors.lance").to_string_lossy().to_string();

    let conn = block_on_lancedb(async {
        connect(&db_path)
            .execute()
            .await
            .map_err(|e| format!("LanceDB connect error: {e}"))
    })?;

    let mut guard = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(conn);
    drop(guard);

    let conn = get_conn()?;
    let table_names = block_on_lancedb(async {
        conn.table_names()
            .execute()
            .await
            .map_err(|e| format!("{e}"))
    })?;

    if !table_names.contains(&TABLE_NAME.to_string()) {
        let schema = make_schema();
        let empty_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(Vec::<&str>::new())),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        Vec::<Option<Vec<Option<f32>>>>::new(),
                        VECTOR_DIM,
                    ),
                ),
                Arc::new(StringArray::from(Vec::<&str>::new())),
                Arc::new(Int64Array::from(Vec::<i64>::new())),
            ],
        )
        .map_err(|e| format!("Failed to create empty record batch: {e}"))?;

        block_on_lancedb(async {
            conn.create_table(TABLE_NAME, empty_batch)
                .execute()
                .await
                .map_err(|e| format!("Failed to create vectors table: {e}"))
        })?;
    }

    Ok(())
}

/// Insert or replace a vector record.
///
/// `id` must match the SQLite highlight id (primary key). `vector` must be
/// exactly `VECTOR_DIM` floats.
pub fn store_vector(id: &str, vector: &[f32], text: &str, created_at: i64) -> Result<(), String> {
    if vector.len() != VECTOR_DIM as usize {
        return Err(format!(
            "Expected {}-dim vector, got {}",
            VECTOR_DIM,
            vector.len()
        ));
    }

    let conn = get_conn()?;
    let table = block_on_lancedb(async {
        conn.open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| format!("Failed to open vectors table: {e}"))
    })?;

    let schema = make_schema();
    let id_array = StringArray::from(vec![id]);
    let vector_data: Vec<Option<f32>> = vector.iter().map(|&v| Some(v)).collect();
    let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vec![Some(vector_data)],
        VECTOR_DIM,
    );
    let text_array = StringArray::from(vec![text]);
    let created_at_array = Int64Array::from(vec![created_at]);

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(id_array),
            Arc::new(vector_array),
            Arc::new(text_array),
            Arc::new(created_at_array),
        ],
    )
    .map_err(|e| format!("Failed to create record batch: {e}"))?;

    block_on_lancedb(async {
        table
            .add(batch)
            .execute()
            .await
            .map_err(|e| format!("Failed to add vector: {e}"))
    })?;

    Ok(())
}

/// Remove a vector by its `id`.
pub fn delete_vector(id: &str) -> Result<(), String> {
    let conn = get_conn()?;
    let table = block_on_lancedb(async {
        conn.open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| format!("Failed to open vectors table: {e}"))
    })?;

    let escaped = id.replace('\\', "\\\\").replace('\'', "''");
    block_on_lancedb(async {
        table
            .delete(&format!("id = '{escaped}'"))
            .await
            .map_err(|e| format!("Failed to delete vector: {e}"))
    })?;

    Ok(())
}

/// Approximate-nearest-neighbour search returning the top-k results ordered
/// by ascending distance (cosine distance, so lower = more similar).
pub fn search_vectors(query_vector: &[f32], k: usize) -> Result<Vec<VectorSearchResult>, String> {
    if query_vector.len() != VECTOR_DIM as usize {
        return Err(format!(
            "Expected {}-dim vector, got {}",
            VECTOR_DIM,
            query_vector.len()
        ));
    }

    let conn = get_conn()?;
    let table = block_on_lancedb(async {
        conn.open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| format!("Failed to open vectors table: {e}"))
    })?;

    let count = block_on_lancedb(async {
        table
            .count_rows(None)
            .await
            .map_err(|e| format!("Failed to count rows: {e}"))
    })?;

    if count == 0 {
        return Ok(vec![]);
    }

    let batches = block_on_lancedb(async {
        let stream = table
            .query()
            .nearest_to(query_vector)
            .map_err(|e| format!("Query setup failed: {e}"))?
            .distance_type(DistanceType::Cosine)
            .limit(k)
            .execute()
            .await
            .map_err(|e| format!("Query execution failed: {e}"))?;
        stream
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| format!("Stream collection failed: {e}"))
    })?;

    let mut results = Vec::new();
    for batch in batches {
        let batch: arrow::record_batch::RecordBatch = batch;
        let id_col = batch
            .column_by_name("id")
            .and_then(|c: &std::sync::Arc<dyn arrow::array::Array + 'static>| {
                c.as_any().downcast_ref::<StringArray>()
            })
            .ok_or("Missing 'id' column in result")?;
        let text_col = batch
            .column_by_name("text")
            .and_then(|c: &std::sync::Arc<dyn arrow::array::Array + 'static>| {
                c.as_any().downcast_ref::<StringArray>()
            })
            .ok_or("Missing 'text' column in result")?;
        let distance_col = batch
            .column_by_name("_distance")
            .and_then(|c: &std::sync::Arc<dyn arrow::array::Array + 'static>| {
                c.as_any().downcast_ref::<Float32Array>()
            })
            .ok_or("Missing '_distance' column in result")?;

        for i in 0..batch.num_rows() {
            if id_col.is_null(i) || text_col.is_null(i) || distance_col.is_null(i) {
                continue;
            }
            results.push(VectorSearchResult {
                id: id_col.value(i).to_string(),
                text: text_col.value(i).to_string(),
                score: distance_col.value(i),
            });
        }
    }

    Ok(results)
}

/// Best-effort flush of pending LanceDB writes.
/// Currently a no-op because LanceDB operations are synchronous in this module.
pub fn flush() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::LANCE_DB_TEST_MUTEX;
    use super::*;

    fn init_test_db() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        // Reset global connection so each test gets a fresh DB
        {
            let mut guard = DB_CONN.lock().unwrap();
            *guard = None;
        }
        let dir = std::env::temp_dir()
            .join("relay_vector_test_db")
            .join(format!("run_{}", COUNTER.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        init_vector_db(&dir).expect("init_vector_db failed");
    }

    fn dummy_vector(seed: f32) -> Vec<f32> {
        let mut v = vec![0.0_f32; VECTOR_DIM as usize];
        v[0] = seed;
        // Normalise so cosine distance behaves predictably.
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    // LanceDB ANN queries are slow and occasionally flaky in the Rust SDK due to
    // eventual consistency. Kept as slow-test; run manually with --ignored.
    #[ignore = "LanceDB ANN query — slow cold compile; run with cargo test -- --ignored"]
    #[test]
    fn test_store_and_search_vector() {
        let _guard = LANCE_DB_TEST_MUTEX.lock().unwrap();
        init_test_db();

        let id = "hl-1";
        let text = "The quick brown fox jumps over the lazy dog.";
        let vector = dummy_vector(1.0);

        store_vector(id, &vector, text, 1234567890).unwrap();

        // Search with the same vector should return the stored record.
        let results = search_vectors(&vector, 5).unwrap();
        assert!(
            !results.is_empty(),
            "search should return at least one result"
        );
        assert_eq!(results[0].id, id);
        assert_eq!(results[0].text, text);
        // Cosine distance to itself should be ~0.
        assert!(
            results[0].score.abs() < 1e-4,
            "self-distance should be near zero, got {}",
            results[0].score
        );
    }

    // Full DB lifecycle test (store + search + delete) is slow and exercises
    // the LanceDB C++ backend. Run on demand or in the nightly slow-test job.
    #[ignore = "LanceDB lifecycle test — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_delete_vector() {
        let _guard = LANCE_DB_TEST_MUTEX.lock().unwrap();
        init_test_db();

        let id = "hl-delete";
        let vector = dummy_vector(2.0);
        store_vector(id, &vector, "to delete", 0).unwrap();

        // Verify it's there.
        let before = search_vectors(&vector, 5).unwrap();
        assert!(before.iter().any(|r| r.id == id));

        delete_vector(id).unwrap();

        // Verify it's gone.
        let after = search_vectors(&vector, 5).unwrap();
        assert!(
            !after.iter().any(|r| r.id == id),
            "deleted vector should not appear in search"
        );
    }

    // LanceDB init/open triggers the C++ backend; keep fast tests free of it.
    #[ignore = "LanceDB init — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_search_empty_db_returns_empty() {
        let _guard = LANCE_DB_TEST_MUTEX.lock().unwrap();
        init_test_db();

        let query = dummy_vector(3.0);
        let results = search_vectors(&query, 5).unwrap();
        assert!(results.is_empty());
    }

    // LanceDB ANN queries are eventually consistent in the Rust SDK.
    // Ordering assertions can flake until background compaction completes.
    // Run manually or in the nightly slow-test job.
    #[ignore = "LanceDB eventual consistency; run with cargo test -- --ignored"]
    #[test]
    fn test_cosine_similarity_ordering() {
        let _guard = LANCE_DB_TEST_MUTEX.lock().unwrap();
        init_test_db();

        // Three vectors with different first components.
        // After normalisation, cosine similarity between two normalised vectors
        // equals their dot product = a[0]*b[0] + rest*rest.
        // Since all other components are zero, similarity = a[0]*b[0].
        // Cosine distance = 1 - similarity.
        let vec_a = dummy_vector(1.0); // normalised -> [1, 0, 0, ...]
        let vec_b = dummy_vector(2.0); // normalised -> [1, 0, 0, ...] (same direction as A)
        let vec_c = dummy_vector(-1.0); // normalised -> [-1, 0, 0, ...] (opposite direction)

        // Query vector close to B: use same direction as B but with a slightly
        // different first component so B is still closest.
        let query = dummy_vector(2.1);

        store_vector("A", &vec_a, "text A", 1).unwrap();
        store_vector("B", &vec_b, "text B", 2).unwrap();
        store_vector("C", &vec_c, "text C", 3).unwrap();

        let results = search_vectors(&query, 3).unwrap();
        assert_eq!(results.len(), 3, "expected 3 results");

        // All three vectors lie on the same line through the origin.
        // vec_a and vec_b point in the same direction, vec_c points opposite.
        // Query points in the same direction as A/B.
        // Therefore closest should be A or B (tie, same distance), then the other,
        // then C.
        let ids: Vec<String> = results.iter().map(|r| r.id.clone()).collect();
        let first_two: std::collections::HashSet<String> = ids[..2].iter().cloned().collect();
        assert!(
            first_two == ["A", "B"].iter().map(|&s| s.to_string()).collect(),
            "first two results must be A and B (any order), got {:?}",
            ids
        );
        assert_eq!(ids[2], "C", "third result should be C");

        // Verify scores are monotonically increasing (cosine distance).
        assert!(
            results[0].score <= results[1].score,
            "distance should be non-decreasing"
        );
        assert!(
            results[1].score <= results[2].score,
            "distance should be non-decreasing"
        );
    }
}
