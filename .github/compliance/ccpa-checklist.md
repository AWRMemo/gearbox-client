# CCPA Compliance Checklist

**Version:** 1.0-beta  
**Effective Date:** 2026-05-25  
**Applies to:** Gearbox Labs, Inc. operations within the State of California, USA  
**Statute:** California Consumer Privacy Act of 2018, as amended by the California Privacy Rights Act (CPRA)

---

## 1. Scope Determination

| Question | Answer |
|----------|--------|
| Does Gearbox buy, sell, or share consumer personal information? | **No.** Gearbox does not sell, rent, or share personal information for monetary or cross-context behavioural advertising purposes. |
| Does Gearbox exceed CCPA/CPRA thresholds? | **N/A** — Gearbox is not a data broker and does not monetise user data. |
| Is Gearbox a "service provider" or "contractor"? | **Yes** — for sync infrastructure hosting, Gearbox acts as a service provider to the end user. |

**Conclusion:** Gearbox Relay is not subject to the "sale" and "sharing" provisions of the CCPA because it does not engage in those activities. However, the consumer rights framework below is implemented as a best-practice matter.

---

## 2. Consumer Rights Mapping

### 2.1 Right to Know (§ 1798.100–110)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Privacy policy discloses categories of personal information collected | ✅ Complete | `docs/privacy-policy.md` §2 |
| Privacy policy discloses purposes of collection | ✅ Complete | `docs/privacy-policy.md` §2 |
| Privacy policy discloses categories of third parties | ✅ Complete | `docs/privacy-policy.md` §4 |
| Method to request specific pieces of information | ✅ Complete | §4 below |
| Method to request categories of information | ✅ Complete | §4 below |

### 2.2 Right to Delete (§ 1798.105)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Mechanism to request deletion of personal information | ✅ Complete | "Clear Data" in Settings; account deletion via email |
| Deletion from service provider systems | ✅ Complete | Account purge within 7 days; automatic GC after 90 days |
| Exceptions documented (e.g. legal hold) | ✅ Complete | §6 below |

### 2.3 Right to Correct (§ 1798.106)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Mechanism to correct inaccurate personal information | ✅ Complete | In-app editing of highlights, tags, and summaries |
| Correction propagated to service providers | ✅ Complete | Next sync push overwrites server blob |

### 2.4 Right to Opt-Out of Sale / Sharing (§ 1798.120)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| "Do Not Sell or Share My Personal Information" link | **N/A** | Gearbox does not sell or share personal information |
| Opt-out preference signal (opt-out link / Global Privacy Control) | ✅ Complete | GPC signal honoured if detected; no action needed because no sale/sharing occurs |

### 2.5 Right to Limit Use of Sensitive Personal Information (§ 1798.121)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Identify sensitive personal information collected | ✅ Complete | No sensitive personal information (SSN, financial account, precise geolocation, health data, etc.) is collected |
| Disclosure of use purposes | ✅ Complete | `docs/privacy-policy.md` §2 |
| Method to limit use | **N/A** | Not applicable — no sensitive PI is processed |

### 2.6 Right to Non-Discrimination (§ 1798.125)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| No discriminatory pricing or quality differences for exercised rights | ✅ Complete | All features available regardless of privacy choices; Free tier value is genuine and unchanged by opt-out |

---

## 3. Business Contact and Disclosure Methods

| Channel | Details | Response SLA |
|---------|---------|-------------|
| **Email** | privacy@gearbox.dev | 72 hours |
| **GitHub Issues** | https://github.com/AWRMemo/gearbox-client/issues | 72 hours |
| **Mail** | Gearbox Labs, Inc. — Privacy Office | 10 business days |

**Website disclosure:** A link to the privacy policy is included in every app build (in-app Settings → Privacy) and on the public repository README.

---

## 4. Verification Process

When a California consumer submits a rights request, Gearbox verifies identity as follows:

| Step | Detail |
|------|--------|
| **4.1 — Request receipt** | Auto-acknowledgment email within 24 hours. |
| **4.2 — Account-holder verification** | Request must originate from the registered email address; a signed-in session token may be required for high-risk requests (deletion). |
| **4.3 — Non-account-holder verification** | If the requester does not have a sync account, Gearbox can only confirm that no server-side data exists (core data is local-first). A signed affidavit may be requested for deletion of account metadata. |
| **4.4 — Agent verification** | If submitted by an authorised agent, Gearbox requires: (a) signed authorisation, (b) identity verification of the consumer, (c) verification of the agent's authority. |
| **4.5 — Response timeline** | 45 days from receipt; extendable by 45 days with written notice. |

---

## 5. Retention

| Data Category | Retention | CCPA Relevance |
|--------------|-----------|----------------|
| Personal information (local) | Until user deletion | User retains full control |
| Encrypted sync blobs | 90 days after acknowledgement | Deletion request honoured within 7 days |
| Telemetry (opt-in) | 90 days | Automatically purged |
| Business records / audit logs | 30 days | Retained for fraud and abuse detection only; no sale/sharing |

---

## 6. Exceptions to Deletion

Pursuant to Cal. Civ. Code § 1798.105(d), Gearbox may retain personal information where necessary to:

1. Complete the transaction for which the personal information was collected.
2. Detect security incidents, protect against malicious, deceptive, fraudulent, or illegal activity.
3. Debug to identify and repair errors.
4. Comply with a legal obligation or exercise/defend legal claims.

Because Gearbox processes only encrypted blobs and scrubbed telemetry, no plaintext user content is retained beyond user control.

---

## 7. Annual Review

This checklist is reviewed **annually** and updated within 30 days of any material change to business practices, data categories, or California law amendments. Last review: 2026-05-25.

---

*This CCPA checklist is maintained as part of the Gearbox Relay Sprint 17 legal and compliance package.*
