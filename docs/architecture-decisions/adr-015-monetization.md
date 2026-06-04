# ADR-015: Monetization Architecture — Stripe + Stripe Connect

## Status
Accepted

## Context
Relay needs to monetize via paid subscriptions (Pro tier) and creator marketplace fees (Creator tier). Sprint 21 implements the full payment infrastructure.

## Decision

### Stripe was chosen over Paddle, LemonSqueezy, and Gumroad
- **Paddle/LemonSqueezy** are Merchant of Record (MoR) options that handle global tax (VAT/GST). They take 5%+ fees vs Stripe's 2.9% + $0.30. More importantly, they lack Stripe Connect for creator payouts — we'd need a separate payout rail.
- **Stripe Checkout** (hosted payment page) means we never touch card data. Zero PCI burden.
- **Stripe Connect Express** enables creator payouts natively. Creators onboard via a Stripe-hosted form. We charge `application_fee_amount` = 15% of each transaction.
- **Stripe Webhooks** (`checkout.session.completed`, `customer.subscription.deleted`, `customer.subscription.updated`) are server-side, signature-verified, and the sole source of truth for tier gating.

### Tier gating is server-authoritative, not client-side
`set_user_tier()` was a Tauri command that any authenticated user could call to self-upgrade. This command is **removed** in Sprint 21. Tier (`free` / `pro`) is now set exclusively by Stripe webhooks on the server. The client reads tier from the server on sync and enforces limits locally.

### Implementation uses raw reqwest, not an SDK
Both `async-stripe` (v1.0.0-rc.5) and `stripe-rust` (v0.12) were evaluated and found unsuitable. `async-stripe` has an unstable API that doesn't expose Checkout Session or Webhook types in its current RC. `stripe-rust` uses a different model. The Stripe REST API is simple enough (3 endpoints: checkout, portal, webhook) that a dedicated SDK adds complexity without benefit.

### Creator tier uses Stripe Connect Express
Creators register via `POST /v1/creators/register` which creates a Stripe Connect Express account and returns an onboarding URL. Webhooks detect completed onboarding and set `is_verified = true`. Only verified creators can monetize streams. Platform fee is 15% of creator subscription revenue.

## Consequences
- Stripe tax compliance (VAT/GST) deferred to Sprint 22 (requires legal review)
- Creator payouts (actual bank transfers) deferred until Stripe Connect is fully activated
- `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET` are required environment variables; server won't start billing without them
- All payment state lives on the server SQLite DB — no external Stripe data store needed
