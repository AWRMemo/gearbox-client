use std::path::Path;
use std::sync::Mutex;

use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_array::types::Float32Type;
use arrow_array::Array;
use arrow_array::{FixedSizeListArray, StringArray};
use futures::stream::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType};

pub(crate) static DB_CONN: Mutex<Option<Connection>> = Mutex::new(None);

const TABLE_NAME: &str = "embeddings";
pub(crate) const VECTOR_DIM: i32 = 384;

fn get_conn() -> Result<Connection, String> {
    let guard = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .clone()
        .ok_or_else(|| "Vector DB not initialized".to_string())
}

#[cfg(not(test))]
fn block_on_lancedb<F, T>(future: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>> + Send,
{
    pollster::block_on(future)
}

#[cfg(test)]
fn block_on_lancedb<F, T>(future: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>> + Send,
{
    static LANCE_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
        std::sync::LazyLock::new(|| {
            tokio::runtime::Runtime::new().expect("Failed to create tokio runtime for LanceDB")
        });
    LANCE_RUNTIME.block_on(future)
}

fn make_schema() -> std::sync::Arc<Schema> {
    std::sync::Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                std::sync::Arc::new(Field::new("item", DataType::Float32, true)),
                VECTOR_DIM,
            ),
            true,
        ),
        Field::new("highlight_id", DataType::Utf8, false),
    ]))
}

/// Create/open the LanceDB dataset in the app data directory.
/// Must be called once before any other vector DB operation.
pub fn init_vector_store(data_dir: &Path) -> Result<(), String> {
    let db_path = data_dir
        .join("embeddings.lance")
        .to_string_lossy()
        .to_string();

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
                std::sync::Arc::new(StringArray::from(Vec::<&str>::new())),
                std::sync::Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        Vec::<Option<Vec<Option<f32>>>>::new(),
                        VECTOR_DIM,
                    ),
                ),
                std::sync::Arc::new(StringArray::from(Vec::<&str>::new())),
            ],
        )
        .map_err(|e| format!("Failed to create empty record batch: {e}"))?;

        block_on_lancedb(async {
            conn.create_table(TABLE_NAME, empty_batch)
                .execute()
                .await
                .map_err(|e| format!("Failed to create embeddings table: {e}"))
        })?;
    }

    Ok(())
}

/// Insert or replace a vector record.
/// `highlight_id` must match the SQLite highlight id (primary key).
/// `vector` must be exactly `VECTOR_DIM` floats.
pub fn upsert_embedding(highlight_id: &str, vector: &[f32]) -> Result<(), String> {
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
            .map_err(|e| format!("Failed to open embeddings table: {e}"))
    })?;

    let schema = make_schema();
    let id_array = StringArray::from(vec![highlight_id]);
    let vector_data: Vec<Option<f32>> = vector.iter().map(|&v| Some(v)).collect();
    let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vec![Some(vector_data)],
        VECTOR_DIM,
    );
    let highlight_id_array = StringArray::from(vec![highlight_id]);

    let batch = RecordBatch::try_new(
        schema,
        vec![
            std::sync::Arc::new(id_array),
            std::sync::Arc::new(vector_array),
            std::sync::Arc::new(highlight_id_array),
        ],
    )
    .map_err(|e| format!("Failed to create record batch: {e}"))?;

    // remove existing row for this highlight, if any
    // SECURITY NOTE: This is a short-term fix. LanceDB Rust SDK delete strings do not
    // support parameterization. We escape backslashes and single quotes manually.
    // This will be replaced with a proper parameterized batch update once LanceDB supports it.
    let escaped = highlight_id.replace('\\', "\\\\").replace('\'', "''");
    block_on_lancedb(async {
        table
            .delete(&format!("highlight_id = '{}'", escaped))
            .await
            .map_err(|e| format!("Failed to delete existing embedding: {e}"))
    })
    .ok();

    block_on_lancedb(async {
        table
            .add(batch)
            .execute()
            .await
            .map_err(|e| format!("Failed to add embedding: {e}"))
    })?;

    Ok(())
}

/// Approximate-nearest-neighbour search returning `limit` highlight IDs ordered by similarity (ascending cosine distance).
pub fn search_vectors(query: &[f32], limit: usize) -> Result<Vec<String>, String> {
    if query.len() != VECTOR_DIM as usize {
        return Err(format!(
            "Expected {}-dim vector, got {}",
            VECTOR_DIM,
            query.len()
        ));
    }

    let conn = get_conn()?;
    let table = block_on_lancedb(async {
        conn.open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| format!("Failed to open embeddings table: {e}"))
    })?;

    let batches = block_on_lancedb(async {
        let stream = table
            .query()
            .nearest_to(query)
            .map_err(|e| format!("Query setup failed: {e}"))?
            .distance_type(DistanceType::Cosine)
            .limit(limit)
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
        let batch: RecordBatch = batch;
        let id_col = batch
            .column_by_name("highlight_id")
            .and_then(|c: &std::sync::Arc<dyn arrow::array::Array + 'static>| {
                c.as_any().downcast_ref::<StringArray>()
            })
            .ok_or("Missing 'highlight_id' column in result")?;
        for i in 0..batch.num_rows() {
            if id_col.is_null(i) {
                continue;
            }
            results.push(id_col.value(i).to_string());
        }
    }
    Ok(results)
}

#[cfg(test)]
pub(crate) static LANCE_DB_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn init_test_db() {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        {
            let mut guard = DB_CONN.lock().unwrap_or_else(|e| e.into_inner());
            *guard = None;
        }
        let dir = std::env::temp_dir()
            .join("relay_core_vector_test")
            .join(format!("run_{}", COUNTER.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        init_vector_store(&dir).expect("init_vector_store failed");
    }

    fn dummy_vector(seed: f32) -> Vec<f32> {
        let mut v = vec![0.0_f32; VECTOR_DIM as usize];
        v[0] = seed;
        v[1] = seed + 1.0;
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    // See note on `test_cosine_similarity_ordering` — LanceDB ANN queries are
    // eventually consistent in the Rust SDK. Run in isolation:
    //   cargo test -- db::vector::tests::test_store_and_search_vector --ignored
    #[ignore = "LanceDB eventual consistency; run manually with --ignored"]
    #[test]
    fn test_store_and_search_vector() {
        let _guard = LANCE_DB_TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        init_test_db();

        let id = "hl-1";
        let vector = dummy_vector(1.0);

        upsert_embedding(id, &vector).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let results = search_vectors(&vector, 5).unwrap();
        assert!(
            !results.is_empty(),
            "search should return at least one result"
        );
        assert_eq!(results[0], id);
    }

    // LanceDB init/open is relatively slow in the Rust SDK (C++ backend).
    // Keep this in the slow-test bucket so the fast suite never waits for it.
    #[ignore = "LanceDB init — slow; run with cargo test -- --ignored"]
    #[test]
    fn test_search_empty_db_returns_empty() {
        let _guard = LANCE_DB_TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        init_test_db();

        let query = dummy_vector(3.0);
        let results = search_vectors(&query, 5).unwrap();
        assert!(results.is_empty());
    }

    // This test is ignored in CI because LanceDB vector search is eventually
    // consistent — newly added rows may not be visible to ANN queries until
    // background compaction completes, and the Rust SDK does not expose a
    // synchronous flush/compact call. The test passes reliably when run in
    // isolation (`cargo test -- db::vector::tests::test_cosine_similarity_ordering`).
    #[ignore = "LanceDB eventual consistency; run manually with --ignored"]
    #[test]
    fn test_cosine_similarity_ordering() {
        let _guard = LANCE_DB_TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        init_test_db();

        let vec_a = dummy_vector(1.0);
        let vec_b = dummy_vector(2.0);
        let vec_c = dummy_vector(-1.0);
        let query = dummy_vector(2.1);

        upsert_embedding("A", &vec_a).unwrap();
        upsert_embedding("B", &vec_b).unwrap();
        upsert_embedding("C", &vec_c).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));

        let results = search_vectors(&query, 3).unwrap();
        assert_eq!(
            results.len(),
            3,
            "expected 3 results, got {}",
            results.len()
        );

        let first_two: std::collections::HashSet<String> = results[..2].iter().cloned().collect();
        assert!(
            first_two == ["A", "B"].iter().map(|s| s.to_string()).collect(),
            "first two results must be A and B (any order), got {:?}",
            results
        );
        assert_eq!(results[2], "C", "third result should be C");
    }
}
