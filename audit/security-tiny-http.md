# tiny_http Loopback Security Audit

**Auditor:** Agent 9 (Sprint 12)
**Scope:** `src-tauri/src/web/server.rs`, `src-tauri/src/commands/stream.rs` (local loopback sharing)
**Date:** 2026-05-23

---

## 1. Binding Verification

**Finding:** The server binds exclusively to `127.0.0.1:0` via `Server::http("127.0.0.1:0")` in `server.rs:18`.

**Status:** PASS

- No configurable host string is exposed.
- `build_stream_url` returns `http://127.0.0.1:{port}/stream/{stream_id}` (`commands/stream.rs:158`).
- The static-file fallback uses the same `public_dir` root; no `0.0.0.0` path exists.

---

## 2. XSS / HTML Escaping

**Finding A (HIGH):** `src-tauri/src/web/mod.rs::generate_stream_page` (used by `commands/stream.rs::generate_stream_html`) interpolates `stream.title`, `stream.description`, and `stream.id` directly into the HTML without escaping. A malicious stream title or description containing HTML/JS will execute in the browser of anyone who opens the generated file or clicks the share link.

**Finding B (HIGH):** Both `web/mod.rs::generate_highlight_html` and `web/server.rs::generate_highlight_html` accept `source_url` and place it into an `<a href="{url}">`. `html_escape` does not alter `javascript:` URLs, so a highlight with `source_url = "javascript:alert(document.cookie)"` produces an active XSS vector via `href`.

**Status:** FAIL — remediated in patch (see below).

**Remediation:**
1. Apply `html_escape()` to `title`, `description`, and `stream_id` in `web/mod.rs::generate_stream_page`.
2. Validate `source_url` schemes: only render as a clickable link if it starts with `http://` or `https://`; otherwise render as plain text.

---

## 3. Content-Security-Policy (CSP)

**Finding (MEDIUM):** Neither `server.rs` nor `mod.rs` injects a CSP `<meta>` tag or HTTP header into the generated HTML. If escaping is bypassed (e.g., via a future regression), CSP would be the last line of defense.

**Status:** FAIL — remediated in patch.

**Remediation:** Add `<meta http-equiv="Content-Security-Policy" content="default-src 'self'; style-src 'unsafe-inline';">` to the `<head>` of every generated stream page. Inline styles are required by the self-contained template; no external resources are loaded.

---

## 4. Path Traversal (Static-File Fallback)

**Finding:** `server.rs` sanitizes `url_path` with `sanitize_path`, which drops empty, `.`, and `..` segments, then verifies `file_path.starts_with(public_dir)`.

**Status:** PASS with caveats.

- The check correctly blocks traversal via `../secret`.
- URL-encoded directory separators (e.g. `%2f`) would remain literal filename characters on the decoded path and are therefore not a traversal risk in this implementation.
- On Windows, if `public_dir` were ever relative, `starts_with` could be bypassed by path case variations or `\?\` prefixes. In practice `public_dir` is always absolute (`config::get_app_dir()`).

---

## 5. CORS Headers

**Finding:** No `Access-Control-Allow-Origin` / `Access-Control-Allow-Methods` headers are emitted. This is correct for a loopback-only origin (`127.0.0.1`), which has no cross-origin traffic by design.

**Status:** PASS

---

## 6. Rate Limiting / DoS

**Finding:** The `tiny_http` loopback has no per-IP or per-request rate limiting. A local attacker (same machine, different user) could flood the port. Because it binds to loopback, remote attackers are excluded.

**Status:** ACCEPTED RISK — local-only scope makes exploitation require local access; mitigation is out of scope for v1.

---

## Summary

| # | Check | Severity | Status |
|---|-------|----------|--------|
| 1 | Binding `127.0.0.1:0` | — | PASS |
| 2 | XSS via unescaped title/description/stream.id | **High** | **FIXED** |
| 3 | XSS via `javascript:` in `source_url` href | **High** | **FIXED** |
| 4 | Missing CSP | **Medium** | **FIXED** |
| 5 | Path Traversal | — | PASS |
| 6 | CORS | — | PASS |
| 7 | Rate Limiting | Low | Accepted Risk |
