# Deploying the relay-sync-server Database to Neon

## What You Need

- A [Neon account](https://neon.tech) (free tier works — 0.5 GB RAM, 0.5 CPU, 512 MB storage)
- The relay-sync-server PostgreSQL schema (see [db.rs](../../relay-sync-server/src/db.rs))

## Step 1 — Create a Neon Project

1. Go to [console.neon.tech](https://console.neon.tech) → **New Project**
2. Name it (e.g., `gearbox-relay`)
3. Region: choose closest to your users (e.g., `US East` for US users)
4. Compute size: **Free** tier is sufficient for startup

## Step 2 — Get Your Connection String

After project creation, go to **Connection Details**:

```text
postgresql://username:password@ep-xxx-xxx-123456.us-east-2.aws.neon.tech/neondb?sslmode=require
```

Copy this — you'll need it for:
1. The Railway environment variable `DATABASE_URL`
2. Local development (optional)

## Step 3 — Update the Schema

The current `db.rs` uses **SQLite**. For Neon (PostgreSQL), you need to either:

### Option A — Use `sqlx` with a PostgreSQL adapter (Recommended for production)

The schema is largely compatible, but:
1. SQLite `datetime('now')` → PostgreSQL `NOW()`
2. SQLite `INTEGER` → PostgreSQL `BIGINT` or `BOOLEAN`
3. Some feature flags differ

### Option B — Keep SQLite for now, migrate later

If you want to ship faster, keep SQLite locally and use the Neon connection string later once you add the PostgreSQL adapter.

## Step 4 — Configure for Neon

### If using `sqlx` with PostgreSQL

In `Cargo.toml`, add:
```toml
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "tls-native-tls"] }
```

Then update `db.rs` to use `sqlx::PgPool` instead of `rusqlite::Connection`.

### For now, use SQLite locally and Neon later

Update your `.env`:
```bash
# Local development (SQLite)
DATABASE_URL=data/relay_sync.db

# Production (Railway + Neon)
# DATABASE_URL=postgresql://username:password@host/dbname?sslmode=require
```

## Step 5 — IP Allowlist (Optional)

Neon's free tier may restrict IPs. To allow Railway's servers:
- In Neon Console → your project → **Connection Settings** → **IP Allowlist**
- Set to **Allow all** (Neon encrypts data in transit; this is acceptable for a sync server)

Alternatively, Neon accepts connections from anywhere when `?sslmode=require` is used.

## Step 6 — Connection String Format for Railway

When adding `DATABASE_URL` to Railway:
```
postgresql://username:password@ep-xxx-xxx-123456.us-east-2.aws.neon.tech/neondb?sslmode=require
```

⚠️ **Important**: The `?sslmode=require` query parameter is required.

## Cost Summary

| Service | Tier | Cost |
|---|---|---|
| Neon | Free | $0 |
| Railway | Starter | $0 (limited hours/month) |
| Railway | usage-based | ~$5–10/month for small sync server |
| **Total** | | **~$5–10/month** |

For hobby/development, both free tiers work fine. For production with decent traffic, expect ~$5–15/month combined.

## Comparison: Neon vs. Railway Managed PostgreSQL

| | Neon | Railway PostgreSQL |
|---|---|---|
| Free tier | 0.5 GB RAM, 512 MB storage | 1 GB RAM, 1 GB storage |
| Branching | Yes (branch your DB) | No |
| SSL | Required (`?sslmode=require`) | Built-in |
| Rust driver | `sqlx` + `tokio-postgres` | Same |
| Setup complexity | Slightly more (connection string) | Easier (plugin) |
| Free forever? | Yes (if under limits) | No (limited hours on free tier) |

**Recommendation:** Use Neon directly — you already have the account, and Neon has better free-tier storage (512 MB vs. 1 GB RAM, but Neon has branching which is valuable for development).