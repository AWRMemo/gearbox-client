# download-qwen-gguf.ps1
# Cross-platform script to download the Qwen 3.5 embedding model with SHA-256 verification.
# Usage: .\scripts\download-qwen-gguf.ps1
# Cache: %LOCALAPPDATA%\relay\models\

#Requires -Version 5.1

$ErrorActionPreference = "Stop"

$MODEL_URL = "https://huggingface.co/Ares-Realm-Studios/Qwen3.5-0.8B-Q4_K_M-GGUF/resolve/main/qwen3.5-0.8b-q4_k_m.gguf"
$MODEL_FILENAME = "qwen3.5-0.8b-q4_k_m.gguf"
$EXPECTED_SHA256 = "0f5b3fd77990f44761275bb8d990d7fa0860c9aa8ad56e0559e402a1f1b03f54"

$MODELS_DIR = Join-Path $env:LOCALAPPDATA "relay\models"
$MODEL_PATH = Join-Path $MODELS_DIR $MODEL_FILENAME
$TEMP_PATH = Join-Path $MODELS_DIR "$MODEL_FILENAME.downloading"

New-Item -ItemType Directory -Force -Path $MODELS_DIR | Out-Null

# Resume + SHA256 check
if (Test-Path $MODEL_PATH) {
    Write-Host "Verifying existing model file..."
    $actualHash = (Get-FileHash -Path $MODEL_PATH -Algorithm SHA256).Hash.ToLower()
    if ($actualHash -eq $EXPECTED_SHA256) {
        Write-Host "Model already present and SHA-256 verified."
        Write-Host $MODEL_PATH
        exit 0
    }
    else {
        Write-Host "SHA-256 mismatch. Re-downloading..."
        Remove-Item $MODEL_PATH -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "Downloading $MODEL_FILENAME from Hugging Face..."
Write-Host "Target: $MODEL_PATH"
Write-Host "Expected SHA-256: $EXPECTED_SHA256"

# Resume partial download
$rangeHeader = @{}
if (Test-Path $TEMP_PATH) {
    $partialSize = (Get-Item $TEMP_PATH).Length
    if ($partialSize -gt 0) {
        Write-Host "Resuming partial download..."
        $rangeHeader["Range"] = "bytes=$partialSize-"
    }
}

$headers = if ($rangeHeader.Count -gt 0) { $rangeHeader } else { $null }

if ($headers) {
    Invoke-WebRequest -Uri $MODEL_URL -UseBasicParsing -Headers $headers -OutFile $TEMP_PATH
}
else {
    Invoke-WebRequest -Uri $MODEL_URL -UseBasicParsing -OutFile $TEMP_PATH
}

# Verify GGUF magic bytes
$bytes = [System.IO.File]::ReadAllBytes($TEMP_PATH)
if ($bytes.Length -lt 4 -or [System.Text.Encoding]::ASCII.GetString($bytes[0..3]) -ne "GGUF") {
    Write-Error "ERROR: Downloaded file does not start with GGUF magic bytes."
    Remove-Item $TEMP_PATH -Force -ErrorAction SilentlyContinue
    exit 1
}

# SHA-256 verification
Write-Host "Verifying SHA-256..."
$actualHash = (Get-FileHash -Path $TEMP_PATH -Algorithm SHA256).Hash.ToLower()
if ($actualHash -ne $EXPECTED_SHA256) {
    Write-Error "ERROR: SHA-256 mismatch.`n  Expected: $EXPECTED_SHA256`n  Actual:   $actualHash"
    Remove-Item $TEMP_PATH -Force -ErrorAction SilentlyContinue
    exit 1
}

Move-Item -Path $TEMP_PATH -Destination $MODEL_PATH -Force
Write-Host "Model downloaded and verified: $MODEL_PATH"
Write-Host $MODEL_PATH
