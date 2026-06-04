#!/usr/bin/env bash
# download-qwen-gguf.sh
# Cross-platform script to download the Qwen 3.5 embedding model with SHA-256 verification.
# Usage: ./scripts/download-qwen-gguf.sh
# Cache: ~/.local/share/relay/models/

set -euo pipefail

MODEL_URL="https://huggingface.co/Ares-Realm-Studios/Qwen3.5-0.8B-Q4_K_M-GGUF/resolve/main/qwen3.5-0.8b-q4_k_m.gguf"
MODEL_FILENAME="qwen3.5-0.8b-q4_k_m.gguf"
EXPECTED_SHA256="0f5b3fd77990f44761275bb8d990d7fa0860c9aa8ad56e0559e402a1f1b03f54"

MODELS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/relay/models"
MODEL_PATH="$MODELS_DIR/$MODEL_FILENAME"
TEMP_PATH="$MODELS_DIR/$MODEL_FILENAME.downloading"

mkdir -p "$MODELS_DIR"

if [ -f "$MODEL_PATH" ]; then
    echo "Verifying existing model file..."
    ACTUAL=$(sha256sum "$MODEL_PATH" | awk '{print $1}')
    if [ "$ACTUAL" = "$EXPECTED_SHA256" ]; then
        echo "Model already present and SHA-256 verified."
        echo "$MODEL_PATH"
        exit 0
    else
        echo "SHA-256 mismatch. Re-downloading..."
        rm -f "$MODEL_PATH"
    fi
fi

echo "Downloading $MODEL_FILENAME from Hugging Face..."
echo "Target: $MODEL_PATH"
echo "Expected SHA-256: $EXPECTED_SHA256"

# Resume support
if [ -f "$TEMP_PATH" ]; then
    PARTIAL_SIZE=$(stat -c%s "$TEMP_PATH" 2>/dev/null || stat -f%z "$TEMP_PATH")
    echo "Resuming partial download..."
    curl -L -f \
        --range "$PARTIAL_SIZE-" \
        -o "$TEMP_PATH" \
        "$MODEL_URL"
else
    curl -L -f \
        -o "$TEMP_PATH" \
        "$MODEL_URL"
fi

# Verify GGUF magic bytes
if ! dd if="$TEMP_PATH" bs=1 count=4 2>/dev/null | grep -q '^GGUF'; then
    echo "ERROR: Downloaded file does not start with GGUF magic bytes."
    rm -f "$TEMP_PATH"
    exit 1
fi

# SHA-256 verification
echo "Verifying SHA-256..."
ACTUAL=$(sha256sum "$TEMP_PATH" | awk '{print $1}')
if [ "$ACTUAL" != "$EXPECTED_SHA256" ]; then
    echo "ERROR: SHA-256 mismatch."
    echo "  Expected: $EXPECTED_SHA256"
    echo "  Actual:   $ACTUAL"
    rm -f "$TEMP_PATH"
    exit 1
fi

mv "$TEMP_PATH" "$MODEL_PATH"
echo "Model downloaded and verified: $MODEL_PATH"
echo "$MODEL_PATH"
