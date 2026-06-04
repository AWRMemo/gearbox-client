#!/usr/bin/env bash
# verify-public-ready.sh — checks that no agent tooling or sensitive content
# exists in git-tracked files that could leak to the public repo.
# Run as a pre-push hook and in CI.
set -euo pipefail

FAILED=0
check() {
    local pattern="$1"
    local message="$2"
    if git ls-files | xargs grep -l "$pattern" -- 2>/dev/null | grep -v '.gitignore' | head -3; then
        echo "BLOCKED: $message"
        echo "  Files matching '$pattern' found in git-tracked content."
        echo "  Move to .kilo/ (gitignored) or remove before pushing."
        FAILED=1
    fi
}

echo "=== Checking for agent tool exposure in tracked files ==="

# Direct file existence checks (regardless of content)
for file in CLAUDE.md .clinerules .cursorrules kilo.json gateway.config.json gateway.env; do
    if git ls-files "$file" | grep -q .; then
        echo "BLOCKED: $file is tracked by git."
        echo "  Run: git rm --cached $file && echo $file >> .gitignore"
        FAILED=1
    fi
done

# Content-based checks
check 'kilo\.json|\.kilo/' "Agent config paths in tracked files"
check '\b(?:agent|Kilo|Hermes)\b.*\b(?:tool|ban|failure|paralleliz|worktree)\b' "Agent tool discussion in tracked docs"
check '\.cursorrules|\.clinerules' "Cursor/Cline config references in tracked files"

# Check that Logo-Brainstorm and similar are NOT tracked
for file in Logo-Brainstorm.md devlog.md; do
    if git ls-files "$file" | grep -q .; then
        echo "BLOCKED: $file is tracked by git. Remove it."
        FAILED=1
    fi
done

# Check sprint plans with agent terminology aren't tracked
for file in docs/sprint-17-plan.md docs/sprint-18-plan.md docs/sprint-19-plan.md; do
    if git ls-files "$file" | grep -q .; then
        echo "BLOCKED: $file is tracked. Move to .kilo/docs/"
        FAILED=1
    fi
done

if [ "$FAILED" -eq 1 ]; then
    echo ""
    echo "Push blocked. Fix the issues above before pushing."
    echo "Agent context files belong in .kilo/ (gitignored directory)."
    echo "Product code and public docs go in tracked files."
    exit 1
fi

echo "Clean. No agent exposure in tracked files."
