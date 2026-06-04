# SOC 2 Type II Readiness Assessment — Stub

**Version:** 0.1-stub  
**Date:** 2026-05-25  
**Scope:** Gearbox Relay sync infrastructure (`relay-sync-server`) and client stewardship  
**Framework:** AICPA Trust Services Criteria (TSC) — Security, Availability, Confidentiality  
**Goal:** Prepare for SOC 2 Type II audit within 12–18 months of beta exit

---

## 1. Executive Summary

This document is a **stub** — it establishes the control-mapping structure and identifies gaps that must be closed before a SOC 2 Type II audit can commence. Gearbox Relay operates a local-first architecture, which materially reduces the scope of server-side controls because end-user data is encrypted client-side and opaque to Gearbox infrastructure.

**In-scope systems:**
- `relay-sync-server` (blob storage, authentication, rate limiting)
- CI/CD pipelines (GitHub Actions)
- Infrastructure access (SSH, secret stores, HSMs)

**Out-of-scope (by architectural design):**
- Client-side local SQLite / LanceDB (under user control)
- On-device AI inference (no server involvement)
- Optional loopback Stream sharing (`127.0.0.1:0`, no server routing)

---

## 2. Trust Services Criteria Mapping

### 2.1 Security (Common Criteria)

| CC # | Control Description | Status | Evidence / Gap |
|------|---------------------|--------|----------------|
| CC6.1 | Logical access security — role-based access, least privilege, MFA | 🟡 Partial | Infrastructure engineers use MFA; RBAC defined but not yet formally documented in IAM policy. |
| CC6.2 | Prior to access, register and authorise users | 🟢 In place | Sync account registration requires email verification; device tokens issued on successful login. |
| CC6.3 | Access credentials are unique and periodically reviewed | 🟡 Partial | Engineer SSH keys are individual; no formal 90-day rotation schedule yet. |
| CC6.6 | Security infrastructure and software — firewalls, IDS, WAF | 🟡 Partial | Cloud provider-managed firewall; no IDS/WAF rules explicitly tested. |
| CC6.7 | Security detection — monitoring of threats and anomalies | 🔴 Gap | No SIEM or formal alerting pipeline. Needs: centralised logging + anomaly detection. |
| CC7.1 | System operations monitoring — audit logs retained and reviewed | 🟡 Partial | Server access logs 30 days; no formal weekly review process. |
| CC7.2 | Incident detection — evaluate anomalies and response | 🔴 Gap | No documented incident response plan (IRP). Needs: IRP + tabletop exercise. |
| CC8.1 | Change management — authorised, tested, and approved changes | 🟡 Partial | CI gates (`cargo clippy`, `cargo test`, `flutter analyze`); no formal CAB for infra changes. |

### 2.2 Availability

| A1.2 | Availability monitoring and system recovery | 🟡 Partial | Uptime not yet instrumented; no formal SLA. Needs: status page + automated alerting. |
| A1.3 | Recovery point objective (RPO) / recovery time objective (RTO) defined | 🔴 Gap | No documented RPO/RTO. Target: RPO = 24h (sync blobs), RTO = 4h. |

### 2.3 Confidentiality

| C1.1 | Identification of confidential information | 🟢 In place | Encrypted blobs are confidential by design; plaintext never enters server boundary. |
| C1.2 | Disposal of confidential data | 🟡 Partial | 90-day GC for acknowledged blobs; no formal secure-erasure (cryptographic shredding) procedure documented. |

---

## 3. Gap Analysis

| Gap ID | Description | Priority | Estimated Effort | Target Quarter |
|--------|-------------|----------|-----------------|----------------|
| SOC-G-01 | Deploy SIEM / centralised logging (e.g. Datadog, Grafana) | P0 | 2–3 weeks | Q3 2026 |
| SOC-G-02 | Document formal Incident Response Plan (IRP) with escalation matrix | P0 | 1 week | Q3 2026 |
| SOC-G-03 | Define and test RPO / RTO with documented failover runbooks | P0 | 2 weeks | Q3 2026 |
| SOC-G-04 | Formalise IAM policy with 90-day credential rotation | P1 | 1 week | Q4 2026 |
| SOC-G-05 | Implement IDS/WAF rules and annual penetration test | P1 | 2–3 weeks | Q4 2026 |
| SOC-G-06 | Establish change advisory board (CAB) for production infra changes | P1 | 1 week | Q4 2026 |
| SOC-G-07 | Add cryptographic shredding procedure for blob disposal | P2 | 3 days | Q1 2027 |
| SOC-G-08 | Publish public status page (e.g. status.gearbox.dev) | P2 | 3 days | Q1 2027 |

---

## 4. Timeline to Audit Readiness

| Phase | Activities | Duration | Estimated Window |
|-------|-----------|----------|----------------|
| **Phase 1 — Foundation** | Close P0 gaps (SOC-G-01 through SOC-G-03); document policies | 3 months | Q3 2026 |
| **Phase 2 — Hardening** | Close P1 gaps (SOC-G-04 through SOC-G-06); third-party pen-test | 3 months | Q4 2026 |
| **Phase 3 — Observation** | Operate controls for a 6-month observation period; collect evidence | 6 months | Q1–Q2 2027 |
| **Phase 4 — Audit** | Engage CPA firm; Type II report issued | 2 months | Q3 2027 |

**Target audit date:** September 2027 (18 months from Sprint 17).

---

## 5. Key Personnel

| Role | Responsibility | Contact |
|------|---------------|---------|
| Security Lead | Owns SOC 2 roadmap, gap closure, vendor management | security@gearbox.dev |
| Infrastructure Lead | Manages IAM, SIEM, backups, failover | infra@gearbox.dev |
| DPO | Privacy policy, DPA, data subject rights | privacy@gearbox.dev |
| Engineering Lead | CI/CD, code integrity, dependency audits | eng@gearbox.dev |

---

## 6. Controls Evidence Inventory

| Control | Evidence Location | Format | Last Updated |
|---------|-------------------|--------|-------------- |
| CI gates passed | `.github/workflows/` | YAML | Sprint 17 |
| Rate limiting implemented | `relay-sync-server/src/rate_limit.rs` | Source code | Sprint 14 |
| Encryption spec | `docs/prd/sync-v2-opaque-blob.md` | Markdown | Sprint 15 |
| Security audit reports | `audit/` | Markdown | Sprint 12–14 |
| DPA template | `docs/dpa-template.md` | Markdown | Sprint 17 |
| Privacy policy | `docs/privacy-policy.md` | Markdown | Sprint 17 |

---

## 7. Notes

- This readiness stub will be promoted to a formal System Description and Control Matrix once P0 gaps are closed.
- The local-first architecture materially reduces SOC 2 scope: server-side controls focus on authentication, availability, and infrastructure security only; confidentiality is largely achieved via client-side encryption.
- Annual penetration testing and vulnerability scanning must be added before Phase 2.

---

*This SOC 2 Type II readiness stub is maintained as part of the Gearbox Relay Sprint 17 legal and compliance package. It is not an attestation report and does not represent an opinion by a CPA firm.*
