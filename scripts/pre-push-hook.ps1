# pre-push-hook.ps1 — blocks push if agent tooling files are tracked by git.
# Install: copy this script to .git/hooks/pre-push (without .ps1 extension)
# or run: echo 'pwsh scripts/pre-push-hook.ps1' > .git/hooks/pre-push

$ErrorActionPreference = "Stop"
$failed = $false

Write-Output "=== Pre-push: checking for agent tool exposure ==="

# Block specific files from being tracked
$blockedFiles = @(
    "CLAUDE.md", ".clinerules", ".cursorrules",
    "kilo.json", "gateway.config.json", "gateway.env",
    "Logo-Brainstorm.md", "devlog.md",
    "docs/sprint-17-plan.md", "docs/sprint-18-plan.md", "docs/sprint-19-plan.md",
    "docs/adr/adr-009-agent-tool-ban.md", "docs/adr/adr-010-sprint-18-recovery.md",
    "docs/architecture-decisions/adr-009-sprint-18-recovery.md",
    "docs/architecture-decisions/adr-010-agent-tool-ban.md"
)

foreach ($file in $blockedFiles) {
    $inGit = git ls-files $file 2>$null
    if ($inGit) {
        Write-Output "BLOCKED: $file is tracked by git (use: git rm --cached $file)"
        $failed = $true
    }
}

# Content checks on tracked files
$trackedFiles = git ls-files 2>$null | Where-Object { $_ -notmatch '\.gitignore|\.kilo/' }
foreach ($file in $trackedFiles) {
    if (-not (Test-Path $file)) { continue }
    $content = Get-Content $file -Raw -ErrorAction SilentlyContinue
    if (-not $content) { continue }

    $basename = Split-Path $file -Leaf
    # Skip binary files and lockfiles
    if ($file -match '\.(png|ico|gguf|lock|downloading)$') { continue }

    # Block agent terminology in tracked docs
    if ($content -match '\b\d+-agent\b' -and $file -match '\.md$') {
        Write-Output "BLOCKED: $file contains AI agent terminology ('N-agent')"
        $failed = $true
    }
    if ($content -match 'agent.*tool.*ban|agent.*failure.*rate|parallel.*agents' -and $file -match '\.md$') {
        Write-Output "BLOCKED: $file discusses AI agent tooling"
        $failed = $true
    }
}

if ($failed) {
    Write-Output ""
    Write-Output "Push blocked. Fix issues above before pushing."
    Write-Output "Agent context files belong in .kilo/ (gitignored)."
    Write-Output "To override (emergency only): git push --no-verify"
    exit 1
}

Write-Output "Clean. Push allowed."
