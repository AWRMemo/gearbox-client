# ADR 012: SM-2 Algorithm for Spaced Repetition Reviews

**Status:** Accepted
**Date:** 2026-05-25

## Context

Themed Review Sessions (PRD §5 Journey 3) require a spaced repetition algorithm to surface highlights for periodic recall. Three candidates were evaluated:

1. **SM-2 (SuperMemo 2)** — public domain, 30+ years of proven results, simple parameter model (ease factor, interval, review count)
2. **FSRS (Free Spaced Repetition Scheduler)** — modern ML-based, requires per-user parameter optimization, complex implementation, no Rust library available
3. **Leitner System** — 5-box physical card model, simple but doesn't adapt to item difficulty

## Decision

Use **SM-2** with canonical parameters from the 2002 specification:

- Initial ease factor: 2.5
- Initial interval: 1 day
- Minimum ease factor: 1.3
- Minimum interval: 1 day
- Grade scale: 0–5 (Again=0, Hard=2, Good=3, Easy=5)
- Grade 0-1 resets interval to 1 day (ease factor unchanged)
- Grade 2-5: new_interval = old_interval × ease_factor, ease_factor adjusted by grade-dependent bonus

Implementation points:
- Algorithm is pure Rust in `relay-core/src/review/mod.rs`
- No external dependencies beyond `chrono` (already in the crate)
- `review_log` table in SQLite stores per-highlight ease factor, interval, next_review_at, review_count
- Desktop UI renders card-flip interface with grade buttons
- Mobile UI shares the same Rust algorithm via FRB (deferred)

## Consequences

- Simple, proven algorithm — no tuning required
- No external ML model or data required
- Each highlight independently tracks its own ease factor and interval
- Grade 0 response resets the interval (forgotten items re-enter rotation quickly)
- FSRS may be evaluated for v2.0 if SM-2 retention is insufficient for power users
