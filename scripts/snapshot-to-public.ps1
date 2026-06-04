# snapshot-to-public.ps1
# Creates a clean public snapshot from the private repo and force-pushes to gearbox-client.
# Run ONLY after verifying the private repo passes scripts/verify-public-ready.sh
#
# What gets stripped:
#   relay-sync-server/      — proprietary server code
#   .kilo/                  — AI agent configuration (gitignored)
#   CLAUDE.md               — agent instruction file
#   kilo.json               — agent config
#   gateway.*               — MCP gateway config
#   .clinerules/.cursorrules — editor AI tool configs
#   fly.toml                — deployment config
#   devlog.md               — internal development log
#   target/ dist/ node_modules/ — build artifacts
#   canned_test_output.txt  — test artifacts
#   quality_bench_output.txt
#   fix_parser.py            — one-off script
#   Logo-Brainstorm.md       — creative exploration
#
# Usage:
#   pwsh scripts/snapshot-to-public.ps1

$ErrorActionPreference = "Stop"
$snapDir = "$env:TEMP\gearbox-snapshot"

Write-Output "Creating clean public snapshot..."

# Clean previous snapshot
if (Test-Path $snapDir) { Remove-Item -Recurse -Force $snapDir }

# Clone private repo
git clone https://github.com/AWRMemo/gearbox.git $snapDir
Set-Location $snapDir

# === Strip proprietary files ===
Remove-Item -Recurse -Force "relay-sync-server" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force ".kilo" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force ".kilocode" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "target" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "dist" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "node_modules" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force ".wrangler" -ErrorAction SilentlyContinue

# === Strip agent tooling configs ===
Remove-Item -Force "CLAUDE.md" -ErrorAction SilentlyContinue
Remove-Item -Force ".clinerules" -ErrorAction SilentlyContinue
Remove-Item -Force ".cursorrules" -ErrorAction SilentlyContinue
Remove-Item -Force "kilo.json" -ErrorAction SilentlyContinue
Remove-Item -Force "gateway.config.json" -ErrorAction SilentlyContinue
Remove-Item -Force "gateway.env" -ErrorAction SilentlyContinue
Remove-Item -Force "Logo-Brainstorm.md" -ErrorAction SilentlyContinue
Remove-Item -Force "devlog.md" -ErrorAction SilentlyContinue

# === Strip deployment configs ===
Remove-Item -Force "fly.toml" -ErrorAction SilentlyContinue

# === Strip test artifacts ===
Remove-Item -Force "canned_test_output.txt" -ErrorAction SilentlyContinue
Remove-Item -Force "quality_bench_output.txt" -ErrorAction SilentlyContinue
Remove-Item -Force "fix_parser.py" -ErrorAction SilentlyContinue

# === Strip agent sprint planning (should be in .kilo/ but belt-and-suspenders) ===
$agentPlans = @(
    "docs/sprint-17-plan.md", "docs/sprint-18-plan.md", "docs/sprint-19-plan.md",
    "docs/adr/adr-009-agent-tool-ban.md", "docs/adr/adr-010-sprint-18-recovery.md",
    "docs/architecture-decisions/adr-009-sprint-18-recovery.md",
    "docs/architecture-decisions/adr-010-agent-tool-ban.md"
)
foreach ($plan in $agentPlans) {
    Remove-Item -Force $plan -ErrorAction SilentlyContinue
}

# === Remove stray files ===
Get-ChildItem -Filter "!" -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue

# === Create clean orphan commit ===
git add -A
git checkout --orphan public-main
git commit -m "Gearbox Relay — public release

A local-first, AI-native personal knowledge pipeline. Capture text highlights,
enrich with on-device AI (Qwen 3.5 GGUF), publish curated Streams.

Stack: Tauri 2.0 (Rust + React), Flutter mobile, relay-core shared library.
License: Apache 2.0"

# === Push to public repo ===
git remote add client https://github.com/AWRMemo/gearbox-client.git
git push client public-main:main --force

Write-Output ""
Write-Output "Public snapshot pushed to github.com/AWRMemo/gearbox-client"
Write-Output "Verify: https://github.com/AWRMemo/gearbox-client"

# Cleanup
Set-Location $env:TEMP
Remove-Item -Recurse -Force $snapDir -ErrorAction SilentlyContinue
