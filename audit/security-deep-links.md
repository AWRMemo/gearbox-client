# Deep-Link Injection Security Audit

**Auditor:** Agent 9 (Sprint 12)
**Scope:** `relay_mobile/lib/services/deep_link_service.dart`, `relay_mobile/lib/main.dart`, `relay_mobile/android/app/src/main/AndroidManifest.xml`, `relay-core/src/db/subscriptions.rs`
**Date:** 2026-05-23

---

## 1. Deep-Link Routing

**Finding A (HIGH — Android):** `AndroidManifest.xml` registers an intent filter for `relay://subscribe/` with `android:scheme="relay"` and `android:host="subscribe"`. It does *not* register `relay://stream/`. This means `relay://stream/{id}` deep links are silently dropped on Android. However, the HTML server generates `href="relay://stream/{stream_id}"` in `web/server.rs` and `web/mod.rs`, and the Dart `DeepLinkService._handle` treats both `stream` and `subscribe` identically.

**Status:** CONSISTENCY BUG — no security risk but broken UX. Remediation: add `relay://stream/{id}` intent filter.

---

## 2. ID Parameter Sanitization

**Finding:** `DeepLinkService._handle` extracts `id = uri.pathSegments[1]` without any sanitization.

File: `relay_mobile/lib/services/deep_link_service.dart:52`

- No regex validation (e.g. UUID format).
- No length truncation.
- The value flows directly to `RelayService.subscribeToStream(streamId)`.

**Status:** MEDIUM risk.

**Exploit path:** A crafted deep link `relay://subscribe/'%22%20OR%20'1'='1` is passed to `subscribeToStream`, which eventually reaches `relay_core::db::subscriptions::subscribe`. The query there is parameterized (`INSERT INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)`), so SQL injection is prevented. However:
- The `stream_id` value is inserted into the local database unvalidated.
- It could be used later in UI rendering or sharing URLs.

**Remediation:** Validate `streamId` length and character set (e.g., alphanumeric-hyphens, max 64 chars) before calling any downstream service.

---

## 3. SSRF / Internal Route Abuse

**Finding (LOW):** Deep links do not carry authentication tokens or session verification before invoking `subscribeToStream`. This means an unauthenticated user who receives a `relay://subscribe/{id}` link can trigger a local DB write. The operation is benign (creates a subscription row tied to the local device ID), but an attacker with physical access to an unlocked phone could force a subscription via a malicious NFC tag or QR code.

**Status:** ACCEPTED RISK — local-only action, no network egress.

---

## 4. iOS Deep-Link Exposure

**Finding:** No iOS Info.plist or entitlements files were found in scope to register the `relay://` scheme. If iOS entitlements are missing, deep links will work only on Android and silently fail on iOS.

**Status:** ESCALATED — requires iOS team to add `CFBundleURLSchemes` entry.

---

## 5. Push Notification Deep-Link Echo

**Finding:** `main.dart` forwards push notifications with `deep_link` payload fields to the router. The payload is not sanitized before `router.go('/subscribe/$streamId')`. If a malicious push payload contains a manipulated `deep_link`, it could trigger unintended navigation.

**Status:** MEDIUM.

**Remediation:** Before using `deep_link` from push payloads, validate the URI scheme and host against an allowlist: `uri.scheme == 'relay' && (uri.host == 'stream' || uri.host == 'subscribe')`.

---

## Summary

| # | Check | Severity | Status |
|---|-------|----------|--------|
| 1 | Android intent filter missing `stream` | Low | Noted / Fix required |
| 2 | No `streamId` length/format validation | **Medium** | **FIXED in `deep_link_service.dart`** |
| 3 | Unauth local DB write via deep link | Low | Accepted Risk |
| 4 | iOS deep-link entitlements missing | Low | Escalated to iOS owner |
| 5 | Push notification deep-link not sanitized | **Medium** | **FIXED in `main.dart`** |

**Code changes applied:**
- `deep_link_service.dart`: Added `streamId` regex validation (`^[a-zA-Z0-9_-]{1,64}$`).
- `main.dart`: Added scheme/host allowlist when processing push notification deep links.
