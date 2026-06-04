# Deploying relay-sync-server to Railway

## Prerequisites
- [Railway account](https://railway.app) (GitHub-connected recommended)
- [Neon PostgreSQL database](#deploying-database-to-neon) provisioned first

## Step 1 — Set Up the Database (Neon)

Follow the [Neon deployment guide](./neon-deployment.md) first. You'll need:
1. A Neon project with a PostgreSQL database
2. The `DATABASE_URL` connection string (format: `postgresql://user:password@host/dbname?sslmode=require`)

## Step 2 — Deploy to Railway

### Option A: GitHub Deploy (Recommended)

1. Go to [dashboard.railway.app](https://dashboard.railway.app) → **New Project** → **Deploy from GitHub repo**
2. Select the `gearbox` repository
3. Railway will auto-detect it's a Rust project

### Option B: Railway CLI

```bash
npm install -g railway
railway login
cd relay-sync-server
railway init
railway up
```

## Step 3 — Configure Environment Variables

In Railway project settings, set:

| Variable | Value | Notes |
|---|---|---|
| `BIND_ADDR` | `0.0.0.0` | Required for Railway's proxy |
| `PORT` | `3000` | Railway sets `$PORT` automatically; this is the internal container port |
| `DATABASE_URL` | `postgresql://...` | From Neon — must include `?sslmode=require` |
| `JWT_SECRET` | Generate with `openssl rand -hex 32` | Must be strong and unique |
| `CORS_ORIGINS` | `https://gearbox.ai,https://www.gearbox.ai` | Production domain(s) |
| `STRIPE_SECRET_KEY` | `sk_live_...` | From Stripe Dashboard → Developers → API keys |
| `STRIPE_WEBHOOK_SECRET` | `whsec_...` | From Stripe Dashboard → Webhooks |
| `STRIPE_PRICE_PRO_MONTHLY` | `price_...` | From Stripe Dashboard → Products |
| `STRIPE_PRICE_PRO_ANNUAL` | `price_...` | From Stripe Dashboard → Products |
| `STRIPE_DOMAIN` | `https://gearbox.ai` | Production frontend domain |

### DATABASE_URL Note for Railway
Railway can provision a PostgreSQL database directly (Project → Add Plugin → PostgreSQL). If you use the Railway-managed DB, copy its connection string. If using Neon, paste the Neon URL as a variable.

## Step 4 — Set Start Command

Railway auto-detects Rust builds, but if needed:
- **Build Command:** `cargo build --release`
- **Start Command:** `./target/release/relay-sync-server`

Set these in: Project → Settings → Start Command.

## Step 5 — Custom Domain (Optional)

1. Project → Settings → Domains → Add domain
2. Add `sync.gearbox.ai` (or subdomain of your choice)
3. Add the DNS record Railway provides (CNAME or A record)
4. Update `CORS_ORIGINS` to include your custom domain

## Step 6 — Verify

```bash
curl https://your-railway-app.up.railway.app/health
# Expected: {"status":"ok"} or similar
```

## Step 7 — Update Frontend

Update the sync server URL in your Tauri frontend config:
- `src-tauri/src/commands/sync.rs` or wherever `SYNC_SERVER_URL` is defined
- Example: `const SYNC_SERVER_URL = "https://sync.gearbox.ai";`

## Troubleshooting

### Build fails
- Railway free tier has memory limits; you may need to add a `railway.toml`:
  ```toml
  [build]
  dockerfile = "Dockerfile"
  ```
- Or build locally and deploy via `railway up --detach`

### 403 CORS errors
- Check `CORS_ORIGINS` exactly matches the frontend origin (no trailing slash)
- Ensure `STRIPE_DOMAIN` matches exactly

### Database connection fails
- Neon requires `?sslmode=require` at the end of `DATABASE_URL`
- Verify the Neon IP allowlist includes Railway's outbound IPs (or use Neon with no IP restrictions — it's encrypted in transit)