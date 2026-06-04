# ADR 011: Public Beta Ships with SEC-12-SYNC-METADATA Known Issue

**Status:** Accepted
**Date:** 2026-05-25

## Context

The sync metadata leak (`SEC-12-SYNC-METADATA`) was identified in Sprint 12 security audit. The OpaqueBlob v2 crypto layer (AES-256-GCM with AAD, no plaintext metadata) is complete and tested (Sprint 14). However, wiring v2 into the SyncEngine requires ~5 days of complex Rust work: dual-protocol push/pull, v1→v2 migration shim, conflict handling for mixed-protocol clients, and server schema migration.

Delaying the public beta until Sprint 20 to fix this would mean:
- 2 additional weeks without real-user feedback
- Risk of building features nobody uses
- Competitors shipping similar products in the gap

The v1 leak is metadata-only (blob IDs, record types, timestamps) — not content, not tags, not summaries.

## Decision

Ship the public beta with `SEC-12-SYNC-METADATA` as a documented known issue. The following mitigations apply:

1. **Documented in known-issues.md** — transparent disclosure to beta users
2. **No content leak** — highlight text, tags, summaries, and connection suggestions remain encrypted
3. **v2 crypto layer is production-ready** — Sprint 20 integration is a known, scoped work item
4. **All future blobs will use v2** — once integrated, old v1 blobs are migrated transparently
5. **Server cannot exploit the leak** — blob IDs are random UUIDs, record types are generic ("highlight" or "stream")

## Consequences

- Public beta ships 2 weeks earlier than otherwise possible
- Beta users accept documented metadata exposure
- Sprint 20 must deliver v2 SyncEngine integration before v1.0 release
- Security-conscious users may defer adoption until v2 is live
- Sprint 20 has a hard deadline: no further delays to OpaqueBlob integration
