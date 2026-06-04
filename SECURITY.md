# Security Policy

## Reporting a Vulnerability

Gearbox Relay markets architectural privacy — we take security reports seriously.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, email: **security@gearbox.dev**

Expect an acknowledgment within 24 hours and a status update within 72 hours. We will coordinate disclosure timing with you and credit you in the release notes (unless you request anonymity).

### What to Include

- Description of the vulnerability and potential impact
- Steps to reproduce (proof-of-concept code welcome)
- Affected components (desktop, mobile, sync server, sync protocol)
- Your preferred contact method and disclosure timeline

### Scope

| Component | In Scope |
|---|---|
| Desktop client (Rust/Tauri) | Yes |
| Mobile client (Flutter) | Yes |
| Sync protocol (OpaqueBlob v2, LWW engine) | Yes |
| Encryption (AES-256-GCM, Argon2id key derivation) | Yes |
| Local HTTP stream server | Yes |
| Sync server (relay-sync-server) | Yes |
| Model download pipeline (GGUF, ONNX) | Yes |
| Telemetry/crash reporting data flow | Yes |
| Chrome extension | Yes |

### Out of Scope

- Theoretical attacks requiring physical device access
- Social engineering
- Denial-of-service against the sync server (rate limiting is best-effort)
- Vulnerabilities in upstream dependencies already patched in our lockfile

### Recognition

We maintain a public [Security Hall of Fame](https://github.com/AWRMemo/gearbox-client#security) for credited researchers. Reports that lead to a fix will be acknowledged in the release changelog.

### PGP Key

```
Coming soon — check back before reporting.
```

---

## Supported Versions

| Version | Supported |
|---|---|
| Sprint 19 beta (current `main`) | Yes |
| Older sprint branches | No |

Only the latest `main` branch receives security patches.
