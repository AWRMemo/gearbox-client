# Beta v1.0 Launch Checklist

**Status:** Sprint 19
**Goal:** Ship public beta with desktop + mobile coverage.

---

## Build Artifacts

- [ ] Desktop: `cargo tauri build` succeeds on Windows, macOS, Linux
- [ ] iOS: `flutter build ios --release --no-codesign` succeeds (compile-check)
- [ ] Android: `flutter build appbundle` succeeds
- [ ] All builds produce versioned artifacts (`v0.1.0`)

## Code Signing

- [ ] Windows: Authenticode certificate configured in CI
- [ ] macOS: Developer ID certificate configured in CI
- [ ] iOS: Distribution certificate + provisioning profile
- [ ] Android: App signing key uploaded to Play Console

## Crash Reporting

- [ ] Desktop Sentry DSN configured (env or SQLite setting)
- [ ] Mobile Sentry DSN configured (env)
- [ ] PII scrubber verified on all platforms (no text, highlights, or user data in events)
- [ ] Telemetry opt-out default (telemetry disabled unless user opts in)

## Privacy & Legal

- [ ] Privacy policy published at docs/privacy-policy.md
- [ ] DPA template available at docs/dpa-template.md
- [ ] CCPA compliance checklist complete
- [ ] GDPR Art.30 records maintained
- [ ] App Store privacy labels drafted

## Server

- [ ] Rate limiting verified (100 req/hr push, 10 req/hr pull)
- [ ] Database backup configured
- [ ] Monitoring / uptime check active
- [ ] `SEC-12-SYNC-METADATA` documented as known issue
- [ ] v2 schema migration path exists but not live

## Onboarding

- [ ] Desktop onboarding modal appears on first launch
- [ ] Mobile onboarding screen appears on first launch
- [ ] "Skip" button on both platforms
- [ ] First capture shows enrichment result

## Error States

- [ ] AI model unavailable → graceful fallback (no crash)
- [ ] Database corruption → reset with diagnostic message
- [ ] Network unavailable → offline mode works
- [ ] Sync conflict → user-visible conflict log

## Accessibility

- [ ] All interactive elements have `aria-label` or visible label (desktop)
- [ ] Focus rings visible on keyboard navigation (desktop)
- [ ] Dark mode toggle works on all screens (desktop)
- [ ] Minimum contrast ratio 4.5:1 for body text (both)

## Dark Mode

- [ ] Desktop: toggle in Settings respects `prefers-color-scheme`
- [ ] Desktop: all 7 screens render correctly in dark mode
- [ ] Mobile: theme system respects system preference
- [ ] Mobile: toggle in Settings persists across restarts

## Mobile

- [ ] iOS build compiles (CI `macos-latest`)
- [ ] Android build compiles (CI `ubuntu-latest`)
- [ ] `flutter analyze` passes with zero errors
- [ ] Background sync: iOS BGAppRefreshTask registered
- [ ] Background sync: Android WorkManager enqueued
- [ ] Android share intent registered

## Beta Invites

- [ ] Invite email template ready (docs/beta-invite-email.md)
- [ ] Feedback form URL configured
- [ ] 10 beta testers selected
- [ ] 1-week feedback window defined

---

## Gate: Go / No-Go

All items above must be checked before tagging `beta-v1.0`.
