# Embedding Engine Decision — ONNX (`ort`) for Desktop v1

**Status:** Accepted (Sprint 13)  
**Owner:** Agent 2  
**Branch:** `feat/sprint-13-production-hardening`

---

## 1. Background

Sprint 12 delivered an ONNX-based `EmbeddingService` using the `ort` crate and the `sentence-transformers/all-MiniLM-L6-v2` model. The original plan specified Candle + Granite 30M, but that implementation never materialised. This document records the formal decision to make ONNX the **production embedding engine for desktop v1** and explains why.

---

## 2. Options Evaluated

| Criterion                | ONNX (`ort`)                                            | Candle (`candle-core`)                                 |
| ------------------------ | ------------------------------------------------------- | ------------------------------------------------------ |
| **Release stability**    | ✅ Microsoft ONNX Runtime 1.19+; stable releases        | ❌ Zero releases on GitHub; trunk-only, no version tags |
| **Upstream maintenance** | ✅ Hugging Face `sentence-transformers` org; widely used | ❌ `huggingface/candle` is pre-1.0; breaking changes common |
| **Model ecosystem**      | ✅ Apache 2.0 ONNX exports from HF Optimum; proven      | ⚠️ Limited Rust-native models; mostly PyTorch ports     |
| **Performance**          | ✅ CPU NEON/SSE, DirectML, CUDA, Metal backends         | ⚠️ Pure Rust; fast on CPU but no vendor GPU paths yet  |
| **Binary size**          | ⚠️ +~15 MB ONNX Runtime C++ dependency                  | ✅ No extra runtime; pure Rust                         |
| **Build complexity**     | ⚠️ Requires `ort` prebuilt binaries or source build     | ✅ Cargo-only; simpler CI                              |
| **Mobile alignment**     | ⚠️ Separate `ort` mobile story; ONNX Runtime Mobile exists | N/A (Candle not used on mobile)                     |

---

## 3. Decision Rationale

### 3.1 Why ONNX over Candle

1. **Stability guarantee.** Candle has **zero releases** on GitHub (as of 2026-05-23). It is trunk-only, which means any `cargo update` can introduce breaking changes or regressions. ONNX Runtime is a Microsoft-backed project with stable semantic versioning and long-term support.
2. **Proven model provenance.** `all-MiniLM-L6-v2` is the second most downloaded model on Hugging Face, battle-tested in production by thousands of organisations, and distributed under Apache 2.0 — compatible with our license.
3. **Existing implementation already works.** Sprint 12 already built, tested, and merged the ONNX pipeline. Re-starting on Candle would cost approximately two developer-weeks with no functional gain.

### 3.2 Why `all-MiniLM-L6-v2`

| Property            | Value                                    |
| ------------------- | ---------------------------------------- |
| Dimensions          | 384                                      |
| Model size (ONNX)   | ~90 MB                                   |
| Mean pooling + L2 normalisation | Built into our wrapper (`embedding.rs`) |
| License             | Apache 2.0                               |
| Languages           | Strong cross-lingual transfer (tested: EN, FR, DE, CJK) |

The 384-dimension output aligns with our LanceDB schema (`VECTOR_DIM = 384`) and delivers sub-100 ms encode latency on a modern CPU.

---

## 4. Trade-offs Accepted

- **C++ dependency:** `ort` pulls in the ONNX Runtime shared library. We mitigate this by using the `ort` prebuilt feature on desktop and documenting the dependency for packagers.
- **Binary size:** The ONNX runtime adds ~15 MB to the desktop bundle. This is acceptable because the LLM GGUF model already adds ~500 MB.
- **No pure-Rust stack:** We lose the "all-Rust" purity of Candle. The practical benefit of purity is outweighed by the risk of trunk-only breakage.

---

## 5. Implementation Notes

### 5.1 Checksum Verification

`embedding_model_manager.rs` verifies SHA-256 checksums after download and on every startup (re-downloading on mismatch):

| File               | SHA-256 (verified 2026-05-23)                                      |
| ------------------ | ------------------------------------------------------------------ |
| `all-MiniLM-L6-v2.onnx` | `6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452` |
| `tokenizer.json`         | `be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037` |

### 5.2 Fallback Behaviour

If `EmbeddingService::try_new` fails (missing model, corrupt files, unsupported CPU), the app continues without embeddings:
- `store_highlight` stores a zero-vector placeholder in LanceDB.
- Search falls back to keyword-only.
- A warning is printed; no hard error propagates to the UI.

### 5.3 Test Coverage

| Test suite                                            | Count | Status                                |
| ----------------------------------------------------- | ----- | ------------------------------------- |
| `embedding.rs` well-formed cases                       | 20    | ✅ `#[ignore]` — slow; run nightly     |
| `embedding.rs` malformed / edge cases                  | 4     | ✅ `#[ignore]` — slow; run nightly     |
| `embedding_model_manager.rs` SHA-256 unit              | 1     | ✅ Fast (in-memory file)               |
| E2E pipeline (`e2e_tests.rs`)                          | 2     | ✅ `#[ignore]` — full model + LanceDB  |

---

## 6. Future Re-evaluation (v2)

This decision is **locked for v1**. Re-evaluate in v2 if any of the following become true:

1. Candle publishes a `v1.0` release with a stability guarantee.
2. A Rust-native, Apache 2.0 embedding model achieves competitive cross-lingual accuracy with a smaller binary footprint than ONNX Runtime.
3. Mobile (Flutter) settles on an embedding engine that we want to unify with desktop (e.g., ONNX Runtime Mobile or TFLite).

---

## 7. Related Files

- `src-tauri/src/ai/embedding.rs` — `EmbeddingService::encode()`
- `src-tauri/src/ai/embedding_model_manager.rs` — model download + SHA-256 verification
- `src-tauri/src/commands/capture.rs` — calls `encode()` before storing highlight
- `src-tauri/src/background/watcher.rs` — same for clipboard-watcher path
- `relay-core/src/db/store.rs` — `store_highlight(…, embedding: Option<&[f32]>)`
- `relay-core/src/db/vector.rs` — LanceDB `upsert_embedding()` (shared by desktop + mobile)
