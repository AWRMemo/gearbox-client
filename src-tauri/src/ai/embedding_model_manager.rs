use ring::digest::{Context, SHA256};
use std::path::{Path, PathBuf};

const HF_BASE: &str = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main";
const ONNX_FILENAME: &str = "all-MiniLM-L6-v2.onnx";
const TOKENIZER_FILENAME: &str = "tokenizer.json";
const ONNX_HF_PATH: &str = "onnx/model.onnx";

// Known-good SHA-256 checksums for the Hugging Face model files.
// Verified 2026-05-23 against sentence-transformers/all-MiniLM-L6-v2.
const ONNX_SHA256: &str = "6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452";
const TOKENIZER_SHA256: &str = "be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037";

fn models_dir(app_dir: &Path) -> PathBuf {
    app_dir.join("models")
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open file for SHA256: {e}"))?;
    let mut ctx = Context::new(&SHA256);
    let mut buf = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)
            .map_err(|e| format!("Failed to read file for SHA256: {e}"))?;
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
    }
    let digest = ctx.finish();
    let actual: String = digest.as_ref().iter().map(|b| format!("{b:02x}")).collect();
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "SHA256 mismatch for {}: expected {expected}, got {actual}. \
             The file may be corrupted or tampered with.",
            path.display()
        ))
    }
}

/// Returns `(model_path, tokenizer_path)` for the embedding pipeline.
/// Downloads both files on first use into `app_dir/models/`.
pub fn ensure_embedding_model(app_dir: &Path) -> Result<(PathBuf, PathBuf), String> {
    let dir = models_dir(app_dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create models directory: {e}"))?;

    let model_path = dir.join(ONNX_FILENAME);
    let tokenizer_path = dir.join(TOKENIZER_FILENAME);

    if model_path.exists() {
        if let Err(e) = verify_sha256(&model_path, ONNX_SHA256) {
            eprintln!("Embedding model checksum failed, re-downloading: {e}");
            let _ = std::fs::remove_file(&model_path);
        }
    }

    if tokenizer_path.exists() {
        if let Err(e) = verify_sha256(&tokenizer_path, TOKENIZER_SHA256) {
            eprintln!("Tokenizer checksum failed, re-downloading: {e}");
            let _ = std::fs::remove_file(&tokenizer_path);
        }
    }

    if !model_path.exists() {
        let url = format!("{HF_BASE}/{ONNX_HF_PATH}");
        download_file(&url, &model_path, "ONNX embedding model", 10_000_000)?;
        verify_sha256(&model_path, ONNX_SHA256).inspect_err(|_| {
            let _ = std::fs::remove_file(&model_path);
        })?;
    }

    if !tokenizer_path.exists() {
        let url = format!("{HF_BASE}/{TOKENIZER_FILENAME}");
        download_file(&url, &tokenizer_path, "tokenizer", 1_000)?;
        verify_sha256(&tokenizer_path, TOKENIZER_SHA256).inspect_err(|_| {
            let _ = std::fs::remove_file(&tokenizer_path);
        })?;
    }

    Ok((model_path, tokenizer_path))
}

fn download_file(
    url: &str,
    dest: &Path,
    label: &str,
    min_expected_bytes: u64,
) -> Result<(), String> {
    let temp_path = dest.with_extension("downloading");

    eprintln!("Downloading {label} from {url}…");

    let result = download_to_temp(url, &temp_path, min_expected_bytes);

    match result {
        Ok(()) => {
            std::fs::rename(&temp_path, dest)
                .map_err(|e| format!("Failed to rename temp file: {e}"))?;
            eprintln!("Download complete: {}", dest.display());
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

fn download_to_temp(url: &str, temp_path: &Path, min_expected_bytes: u64) -> Result<(), String> {
    let response =
        reqwest::blocking::get(url).map_err(|e| format!("Failed to start download: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with HTTP {}: expected a model file but got an error response",
            response.status()
        ));
    }

    let total = response.content_length().unwrap_or(min_expected_bytes);
    if total < min_expected_bytes {
        return Err(format!(
            "Downloaded file is only {} bytes, expected at least {} bytes. The URL may be returning an error page.",
            total,
            min_expected_bytes
        ));
    }

    let mut file =
        std::fs::File::create(temp_path).map_err(|e| format!("Failed to create temp file: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut last_percent = 0u32;

    let mut reader = std::io::BufReader::new(response);
    use std::io::{BufRead, Write};

    loop {
        let buf = reader.fill_buf().map_err(|e| format!("Read error: {e}"))?;
        if buf.is_empty() {
            break;
        }
        let len = buf.len();
        file.write_all(buf)
            .map_err(|e| format!("Write error: {e}"))?;
        reader.consume(len);
        downloaded += len as u64;

        let percent = ((downloaded as f64 / total as f64) * 100.0) as u32;
        if percent != last_percent && percent.is_multiple_of(10) {
            eprintln!(
                "Download progress: {}% ({} MB / {} MB)",
                percent,
                downloaded / 1_000_000,
                total / 1_000_000
            );
            last_percent = percent;
        }
    }

    file.flush().map_err(|e| format!("Flush error: {e}"))?;
    drop(file);

    Ok(())
}
